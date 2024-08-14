use crate::application::services::deploy_service::{DeployService, DeployServiceImpl};
use crate::domain::path::PathOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use std::path::{Path, PathBuf};
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
    PathOperations {}
    #[async_trait]
    impl PathOperations for PathOperations {
        async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn get_home_dir(&self) -> Result<PathBuf, AppError>;
    }
}

#[tokio::test]
async fn test_deploy_execute() {
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_path = MockPathOperations::new();

    mock_shell
        .expect_output()
        .with(eq("cargo build --release"))
        .returning(|_| {
            Ok(std::process::Output {
                status: std::process::ExitStatus::from_raw(0),
                stdout: vec![],
                stderr: vec![],
            })
        });

    mock_shell
        .expect_execute()
        .returning(|_| Ok("Command executed successfully".to_string()));

    mock_path
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    let deploy_service = DeployServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_path));

    let result = deploy_service.execute().await;
    assert!(result.is_ok());
}
