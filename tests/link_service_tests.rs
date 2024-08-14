use crate::application::services::link_service::{LinkService, LinkServiceImpl};
use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

mock! {
    PathOperations {}
    #[async_trait]
    impl PathOperations for PathOperations {
        async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn get_home_dir(&self) -> Result<PathBuf, AppError>;
    }
}

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
