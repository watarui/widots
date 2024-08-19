use crate::domain::path::PathOperations;
use crate::error::AppError;
use async_trait::async_trait;
use dirs::home_dir;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PathExpander;

impl Default for PathExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl PathExpander {
    pub fn new() -> Self {
        PathExpander
    }
}

#[async_trait]
impl PathOperations for PathExpander {
    async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = if path.starts_with("~") {
            self.get_home_dir().await?.join(
                path.strip_prefix("~")
                    .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
            )
        } else {
            path.to_path_buf()
        };
        Ok(expanded_path)
    }

    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = self.expand_tilde(path).await?;

        if !expanded_path.exists() {
            return Ok(expanded_path);
        }
        expanded_path
            .canonicalize()
            .map_err(|_| AppError::DirectoryNotFound)
    }

    async fn get_home_dir(&self) -> Result<PathBuf, AppError> {
        home_dir().ok_or(AppError::DirectoryNotFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    mock! {
        pub PathExpander {}

        #[async_trait]
        impl PathOperations for PathExpander {
            async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
            async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
            async fn get_home_dir(&self) -> Result<PathBuf, AppError>;
        }
    }

    #[test]
    fn test_toml_path_expander_default() {
        let default_parser = PathExpander;
        let new_parser = PathExpander::new();

        // Ensure that the default implementation works correctly
        assert_eq!(format!("{:?}", default_parser), format!("{:?}", new_parser));
    }

    #[tokio::test]
    async fn test_expand_tilde() -> Result<(), AppError> {
        let path_expander = PathExpander::new();
        let result = path_expander.expand_tilde(Path::new("~/test")).await?;
        assert!(result.to_str().unwrap().contains("/test"));
        assert!(!result.to_str().unwrap().contains('~'));
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_path() -> Result<(), AppError> {
        let path_expander = PathExpander::new();
        let result = path_expander.parse_path(Path::new("/tmp/test")).await?;
        assert!(result.starts_with("/"));
        assert!(result.ends_with("tmp/test"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_home_dir() -> Result<(), AppError> {
        let path_expander = PathExpander::new();
        let result = path_expander.get_home_dir().await?;
        assert!(result.is_absolute());
        Ok(())
    }

    fn arbitrary_path_component() -> BoxedStrategy<String> {
        prop::string::string_regex("[a-zA-Z0-9_-]{1,10}")
            .unwrap()
            .boxed()
    }

    proptest! {
        #[test]
        fn test_expand_tilde_idempotent(
            path in prop::collection::vec(arbitrary_path_component(), 0..5)
                .prop_map(|components| PathBuf::from("~").join(components.join("/")))
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let path_expander = PathExpander::new();
                let expanded_once = path_expander.expand_tilde(&path).await.unwrap();
                let expanded_twice = path_expander.expand_tilde(&expanded_once).await.unwrap();

                prop_assert_eq!(expanded_once, expanded_twice);
                Ok(())
            }).unwrap();
        }

        #[test]
        fn test_parse_path_idempotent(
            path in prop::collection::vec(arbitrary_path_component(), 0..5)
                .prop_map(|components| PathBuf::from("/").join(components.join("/")))
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let path_expander = PathExpander::new();
                let parsed_once = path_expander.parse_path(&path).await.unwrap();
                let parsed_twice = path_expander.parse_path(&parsed_once).await.unwrap();

                prop_assert_eq!(parsed_once, parsed_twice);
                Ok(())
            }).unwrap();
        }
    }

    #[tokio::test]
    async fn test_expand_tilde_with_mock() {
        let mut mock = MockPathExpander::new();
        mock.expect_expand_tilde()
            .with(eq(Path::new("~/test")))
            .times(1)
            .returning(|_| Ok(PathBuf::from("/home/user/test")));

        let result = mock.expand_tilde(Path::new("~/test")).await;
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/test"));
    }

    #[tokio::test]
    async fn test_parse_path_with_mock() {
        let mut mock = MockPathExpander::new();
        mock.expect_parse_path()
            .with(eq(Path::new("/tmp/test")))
            .times(1)
            .returning(|_| Ok(PathBuf::from("/tmp/test")));

        let result = mock.parse_path(Path::new("/tmp/test")).await;
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test"));
    }

    #[tokio::test]
    async fn test_get_home_dir_error() {
        let mut mock = MockPathExpander::new();
        mock.expect_get_home_dir()
            .times(1)
            .returning(|| Err(AppError::DirectoryNotFound));

        let result = mock.get_home_dir().await;
        assert!(matches!(result, Err(AppError::DirectoryNotFound)));
    }

    fn arb_tilde_path() -> impl Strategy<Value = PathBuf> {
        "[a-zA-Z0-9_/.]{0,20}".prop_map(|s| PathBuf::from(format!("~/{}", s)))
    }

    fn arb_absolute_path() -> impl Strategy<Value = PathBuf> {
        "[a-zA-Z0-9_/.]{0,20}".prop_map(|s| PathBuf::from(format!("/{}", s)))
    }

    proptest! {
        #[test]
        fn test_expand_tilde_with_various_inputs(path in arb_tilde_path()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let expander = PathExpander::new();
                let result = expander.expand_tilde(&path).await;
                prop_assert!(result.is_ok(), "expand_tilde failed");
                let expanded = result.unwrap();
                prop_assert!(!expanded.to_str().unwrap().contains('~'), "expanded path still contains tilde");
                prop_assert!(expanded.is_absolute(), "expanded path is not absolute");
                Ok(())
            }).unwrap();
        }

        #[test]
        fn test_parse_path_with_various_inputs(path in arb_absolute_path()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let expander = PathExpander::new();
                let result = expander.parse_path(&path).await;
                prop_assert!(result.is_ok(), "parse_path failed");
                Ok(())
            }).unwrap();
        }
    }
}
