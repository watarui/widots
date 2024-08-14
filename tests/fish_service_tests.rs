use crate::application::services::fish_service::{FishService, FishServiceImpl};
use crate::domain::os::OSOperations;
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

mock! {
    OSOperations {}
    #[async_trait]
    impl OSOperations for OSOperations {
        async fn get_os(&self) -> Result<String, AppError>;
    }
}

#[tokio::test]
async fn test_fish_install() {
    let mut mock_shell = MockShellExecutor::new();
    let mock_fs = MockFileSystemOperations::new();
    let mut mock_os = MockOSOperations::new();

    mock_os
        .expect_get_os()
        .returning(|| Ok("macos".to_string()));

    mock_shell
        .expect_execute()
        .with(eq("brew install fish"))
        .returning(|_| Ok("Fish installed successfully".to_string()));

    let fish_service =
        FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

    let result = fish_service.install().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fish_set_default() {
    let mut mock_shell = MockShellExecutor::new();
    let mock_fs = MockFileSystemOperations::new();
    let mock_os = MockOSOperations::new();

    mock_shell
        .expect_execute()
        .returning(|_| Ok("Command executed successfully".to_string()));

    let fish_service =
        FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

    let result = fish_service.set_default().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fish_install_fisher() {
    let mut mock_shell = MockShellExecutor::new();
    let mock_fs = MockFileSystemOperations::new();
    let mock_os = MockOSOperations::new();

    mock_shell
        .expect_execute()
        .returning(|_| Ok("Fisher installed successfully".to_string()));

    let fish_service =
        FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

    let result = fish_service.install_fisher().await;
    assert!(result.is_ok());
}
