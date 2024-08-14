use crate::domain::path::PathOperations;
use crate::error::AppError;
use async_trait::async_trait;
// use dirs::home_dir;
use std::path::{Path, PathBuf};

pub struct PathExpander;

impl PathExpander {
    pub fn new() -> Self {
        PathExpander
    }
}

#[async_trait]
impl PathOperations for PathExpander {
    async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = path.to_path_buf();
        // let expanded_path = if path.starts_with("~") {
        //     self.get_home_dir().await?.join(
        //         path.strip_prefix("~")
        //             .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
        //     )
        // } else {
        //     path.to_path_buf()
        // };
        Ok(expanded_path)
    }

    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = self.expand_tilde(path).await?;

        if !expanded_path.exists() {
            return Ok(expanded_path);
        }
        Ok(expanded_path)
        // expanded_path
        //     .canonicalize()
        //     .map_err(|_| AppError::DirectoryNotFound)
    }

    // async fn get_home_dir(&self) -> Result<PathBuf, AppError> {
    //     home_dir().ok_or(AppError::DirectoryNotFound)
    // }
}

// #[tokio::test]
// async fn test_expand_tilde() -> Result<(), AppError> {
//     let path_expander = PathExpander::new();
//     let result = path_expander.expand_tilde(Path::new("~/test")).await?;
//     assert!(result.to_str().unwrap().contains("/test"));
//     assert!(!result.to_str().unwrap().contains("~"));
//     Ok(())
// }

// #[tokio::test]
// async fn test_parse_path() -> Result<(), AppError> {
//     let path_expander = PathExpander::new();
//     let result = path_expander.parse_path(Path::new("/tmp/test")).await?;
//     assert_eq!(result, Path::new("/tmp/test"));
//     Ok(())
// }

// #[tokio::test]
// async fn test_get_home_dir() -> Result<(), AppError> {
//     let path_expander = PathExpander::new();
//     let result = path_expander.get_home_dir().await?;
//     assert!(result.is_absolute());
//     Ok(())
// }
