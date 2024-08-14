use crate::application::services::load_service::{LoadService, LoadServiceImpl};
use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::models::config::Config;
use crate::models::link::FileProcessResult;
use crate::utils::toml::TomlOperations;
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
    TomlOperations {}
    #[async_trait]
    impl TomlOperations for TomlOperations {
        async fn parse(&self, path: &Path) -> Result<Config, AppError>;
    }
}

mock! {
    OSOperations {}
    #[async_trait]
    impl OSOperations for OSOperations {
        async fn get_os(&self) -> Result<String, AppError>;
    }
}

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
    PromptOperations {}
    #[async_trait]
    impl PromptOperations for PromptOperations {
        async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
    }
}

#[tokio::test]
async fn test_load() {
    let mut mock_link_ops = MockLinkOperations::new();
    let mut mock_path_ops = MockPathOperations::new();
    let mut mock_toml_ops = MockTomlOperations::new();
    let mut mock_os_ops = MockOSOperations::new();
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_prompt_ops = MockPromptOperations::new();

    mock_path_ops
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    mock_toml_ops
        .expect_parse()
        .returning(|_| Ok(Config::default()));

    mock_os_ops
        .expect_get_os()
        .returning(|| Ok("macos".to_string()));

    mock_prompt_ops
        .expect_confirm_action()
        .returning(|_| Ok(true));

    mock_link_ops
        .expect_link_recursively()
        .returning(|_, _, _| Ok(vec![]));

    let load_service = LoadServiceImpl::new(
        Arc::new(mock_link_ops),
        Arc::new(mock_path_ops),
        Arc::new(mock_toml_ops),
        Arc::new(mock_os_ops),
        Arc::new(mock_shell),
        Arc::new(mock_prompt_ops),
    );

    let result = load_service
        .load(Path::new("/config.toml"), Path::new("/target"), false)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_with_provision() {
    let mut mock_link_ops = MockLinkOperations::new();
    let mut mock_path_ops = MockPathOperations::new();
    let mut mock_toml_ops = MockTomlOperations::new();
    let mut mock_os_ops = MockOSOperations::new();
    let mut mock_shell = MockShellExecutor::new();
    let mut mock_prompt_ops = MockPromptOperations::new();

    mock_path_ops
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    mock_toml_ops.expect_parse().returning(|_| {
        Ok(Config {
            provision: Some(vec![crate::models::config::Provision {
                mode: "macos".to_string(),
                script: "echo 'Hello, World!'".to_string(),
            }]),
            ..Default::default()
        })
    });

    mock_os_ops
        .expect_get_os()
        .returning(|| Ok("macos".to_string()));

    mock_prompt_ops
        .expect_confirm_action()
        .returning(|_| Ok(true));

    mock_shell
        .expect_execute()
        .returning(|_| Ok("Provision executed successfully".to_string()));

    let load_service = LoadServiceImpl::new(
        Arc::new(mock_link_ops),
        Arc::new(mock_path_ops),
        Arc::new(mock_toml_ops),
        Arc::new(mock_os_ops),
        Arc::new(mock_shell),
        Arc::new(mock_prompt_ops),
    );

    let result = load_service
        .load(Path::new("/config.toml"), Path::new("/target"), false)
        .await;

    assert!(result.is_ok());
}
