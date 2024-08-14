use crate::application::services::vscode_service::{VSCodeService, VSCodeServiceImpl};
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use std::path::Path;
use std::sync::Arc;

mock! {
    ShellExecutor {}
    #[async_trait]
    impl ShellExecutor for ShellExecutor {
        async fn execute(&self, command: &str) -> Result<String, AppError>;
        async fn output(&self, command: &str) -> Result<std::process::Output, AppError>;
        fn stderr(&self, output: &std::process::Output) -> String;
    }
}

mock! {
    FileSystemOperations {}
    #[async_trait]
    impl FileSystemOperations for FileSystemOperations {
        async fn read_lines(&self, path: &Path) -> Result<Vec<String>, AppError>;
        async fn write_lines(&self, path: &Path, lines: &[String]) -> Result<(), AppError>;
    }
}

#[tokio::test]
async fn test_export_extensions() {
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_fs = MockFileSystemOperations::new();

    mock_shell
        .expect_execute()
        .with(eq("code --list-extensions"))
        .returning(|_| Ok("extension1\nextension2".to_string()));

    mock_fs.expect_write_lines().returning(|_, _| Ok(()));

    let vscode_service = VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

    let result = vscode_service.export_extensions().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_import_extensions() {
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_fs = MockFileSystemOperations::new();

    mock_fs
        .expect_read_lines()
        .returning(|_| Ok(vec!["extension1".to_string(), "extension2".to_string()]));

    mock_shell
        .expect_execute()
        .returning(|_| Ok("Extension installed successfully".to_string()));

    let vscode_service = VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

    let result = vscode_service.import_extensions().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ensure_code_command() {
    let mut mock_shell = MockShellExecutor::new();
    let mock_fs = MockFileSystemOperations::new();

    mock_shell
        .expect_execute()
        .with(eq("which code"))
        .returning(|_| Ok("/usr/local/bin/code".to_string()));

    let vscode_service = VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

    let result = vscode_service.ensure_code_command().await;
    assert!(result.is_ok());
}
