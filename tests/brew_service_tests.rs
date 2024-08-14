use crate::application::services::brew_service::{BrewService, BrewServiceImpl};
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
async fn test_brew_install() {
    let mut mock_shell = MockShellExecutor::new();
    let mock_fs = MockFileSystemOperations::new();

    mock_shell
        .expect_execute()
        .with(eq("/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""))
        .returning(|_| Ok("Homebrew installed successfully".to_string()));

    let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

    let result = brew_service.install().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_brew_import() {
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_fs = MockFileSystemOperations::new();

    mock_fs
        .expect_read_lines()
        .returning(|_| Ok(vec!["package1".to_string(), "package2".to_string()]));

    mock_shell
        .expect_execute()
        .returning(|_| Ok("Package installed successfully".to_string()));

    let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

    let result = brew_service.import().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_brew_export() {
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_fs = MockFileSystemOperations::new();

    mock_shell
        .expect_execute()
        .returning(|_| Ok("package1\npackage2".to_string()));

    mock_fs.expect_write_lines().returning(|_, _| Ok(()));

    let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

    let result = brew_service.export().await;
    assert!(result.is_ok());
}
