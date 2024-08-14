use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
#[cfg(test)]
use mockall::mock;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::sync::Arc;

#[async_trait]
pub trait LinkService: Send + Sync {
    async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError>;
    async fn materialize_dotfiles(&self, target: &Path)
        -> Result<Vec<FileProcessResult>, AppError>;
}

pub struct LinkServiceImpl {
    link_operations: Arc<dyn LinkOperations>,
    path_operations: Arc<dyn PathOperations>,
    prompter: Arc<dyn PromptOperations>,
}

impl LinkServiceImpl {
    pub fn new(
        link_operations: Arc<dyn LinkOperations>,
        path_operations: Arc<dyn PathOperations>,
        prompter: Arc<dyn PromptOperations>,
    ) -> Self {
        Self {
            link_operations,
            path_operations,
            prompter,
        }
    }
}

#[async_trait]
impl LinkService for LinkServiceImpl {
    async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let source = self.path_operations.parse_path(source).await?;
        let target = self.path_operations.parse_path(target).await?;

        let ans = self
            .prompter
            .confirm_action(&format!(
                "This will link files from {:?} to {:?}. Do you want to continue?",
                source.display(),
                target.display()
            ))
            .await?;
        if !ans {
            return Ok(vec![]);
        }

        self.link_operations
            .link_recursively(&source, &target, force)
            .await
    }

    async fn materialize_dotfiles(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let target = self.path_operations.parse_path(target).await?;

        if !self
            .prompter
            .confirm_action(&format!(
                "This will materialize symlinks in {:?}. Do you want to continue?",
                target.display()
            ))
            .await?
        {
            return Ok(vec![]);
        }

        self.link_operations
            .materialize_symlinks_recursively(&target)
            .await
    }
}

#[cfg(test)]
mock! {
    LinkOperations {}
    #[async_trait]
    impl LinkOperations for LinkOperations {
        async fn link_recursively(
            &self,
            source: &Path,
            target: &Path,
            force: bool,
        ) -> Result<Vec<FileProcessResult>, AppError>;
        async fn materialize_symlinks_recursively(
            &self,
            target: &Path,
        ) -> Result<Vec<FileProcessResult>, AppError>;
        fn should_ignore(&self, path: &Path) -> bool;
    }
}

#[cfg(test)]
mock! {
    PathOperations {}
    #[async_trait]
    impl PathOperations for PathOperations {
        async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn get_home_dir(&self) -> Result<PathBuf, AppError>;
    }
}

#[cfg(test)]
mock! {
    PromptOperations {}
    #[async_trait]
    impl PromptOperations for PromptOperations {
        async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
    }
}

#[tokio::test]
async fn test_link_dotfiles() {
    let mut mock_link_ops = MockLinkOperations::new();
    let mut mock_path_ops = MockPathOperations::new();
    let mut mock_prompt_ops = MockPromptOperations::new();

    mock_path_ops
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    mock_prompt_ops
        .expect_confirm_action()
        .returning(|_| Ok(true));

    mock_link_ops
        .expect_link_recursively()
        .returning(|_, _, _| {
            Ok(vec![
                FileProcessResult::Linked(
                    PathBuf::from("/source/file1"),
                    PathBuf::from("/target/file1"),
                ),
                FileProcessResult::Created(PathBuf::from("/target/dir1")),
            ])
        });

    let link_service = LinkServiceImpl::new(
        Arc::new(mock_link_ops),
        Arc::new(mock_path_ops),
        Arc::new(mock_prompt_ops),
    );

    let result = link_service
        .link_dotfiles(Path::new("/source"), Path::new("/target"), false)
        .await;

    assert!(result.is_ok());
    let file_results = result.unwrap();
    assert_eq!(file_results.len(), 2);
}

#[tokio::test]
async fn test_materialize_dotfiles() {
    let mut mock_link_ops = MockLinkOperations::new();
    let mut mock_path_ops = MockPathOperations::new();
    let mut mock_prompt_ops = MockPromptOperations::new();

    mock_path_ops
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    mock_prompt_ops
        .expect_confirm_action()
        .returning(|_| Ok(true));

    mock_link_ops
        .expect_materialize_symlinks_recursively()
        .returning(|_| {
            Ok(vec![FileProcessResult::Materialized(
                PathBuf::from("/target/file1"),
                PathBuf::from("/source/file1"),
            )])
        });

    let link_service = LinkServiceImpl::new(
        Arc::new(mock_link_ops),
        Arc::new(mock_path_ops),
        Arc::new(mock_prompt_ops),
    );

    let result = link_service
        .materialize_dotfiles(Path::new("/target"))
        .await;

    assert!(result.is_ok());
    let file_results = result.unwrap();
    assert_eq!(file_results.len(), 1);
}
