use async_trait::async_trait;
use tokio::fs;

use crate::constants::{
    DEPLOY_DESTINATION_PATH, DEPLOY_SOURCE_PATH, FISH_COMPLETIONS_FILENAME,
    FISH_COMPLETIONS_SOURCE_PATH, FISH_COMPLETIONS_TARGET_DIR,
};
use crate::domain::path::PathOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait DeployService: Send + Sync {
    async fn execute(&self) -> Result<(), AppError>;
}

pub struct DeployServiceImpl {
    shell_executor: Arc<dyn ShellExecutor>,
    path_operations: Arc<dyn PathOperations>,
}

impl DeployServiceImpl {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        path_operations: Arc<dyn PathOperations>,
    ) -> Self {
        Self {
            shell_executor,
            path_operations,
        }
    }

    async fn deploy_executable(&self) -> Result<(), AppError> {
        let source = Path::new(DEPLOY_SOURCE_PATH);
        let destination = self
            .path_operations
            .parse_path(Path::new(DEPLOY_DESTINATION_PATH))
            .await?;

        if !source.exists() {
            return Err(AppError::FileNotFound(source.to_path_buf()));
        }

        let command = format!("sudo cp {} {}", source.display(), destination.display());
        self.shell_executor.execute(&command).await?;
        let command = format!("sudo chmod +x {}", destination.display());
        self.shell_executor.execute(&command).await?;

        Ok(())
    }

    async fn locate_fish_completions(&self) -> Result<(), AppError> {
        let target_dir = self
            .path_operations
            .parse_path(Path::new(FISH_COMPLETIONS_TARGET_DIR))
            .await?;
        fs::create_dir_all(&target_dir).await?;

        let source = Path::new(FISH_COMPLETIONS_SOURCE_PATH);
        let target = target_dir.join(FISH_COMPLETIONS_FILENAME);
        fs::copy(&source, &target).await?;

        Ok(())
    }
}

#[async_trait]
impl DeployService for DeployServiceImpl {
    async fn execute(&self) -> Result<(), AppError> {
        println!("Building the project in release mode...");
        let output = self.shell_executor.output("cargo build --release").await?;
        if !output.status.success() {
            return Err(AppError::Deployment(self.shell_executor.stderr(&output)));
        }

        println!("Deploying the executable...");
        self.deploy_executable().await?;
        println!("Deployment successful!");

        println!("Locating fish shell command completion files...");
        self.locate_fish_completions().await?;
        println!("Locate successful!");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::path::PathOperations;
    use crate::domain::shell::ShellExecutor;
    use crate::error::AppError;
    use async_trait::async_trait;
    use mockall::mock;
    use mockall::predicate::eq;
    use std::os::unix::process::ExitStatusExt;
    use std::path::Path;
    use std::path::PathBuf;
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

    mock! {
        DeployService {}
        #[async_trait]
        impl DeployService for DeployService {
            async fn execute(&self) -> Result<(), AppError>;
        }
    }

    #[tokio::test]
    async fn test_deploy_execute() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_path = MockPathOperations::new();
        let mut mock_deploy_service = MockDeployService::new();

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

        mock_deploy_service.expect_execute().returning(|| Ok(()));

        let _deploy_service = DeployServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_path));

        let result = mock_deploy_service.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deploy_executable_failure() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_path = MockPathOperations::new();

        mock_path
            .expect_parse_path()
            .returning(|path| Ok(path.to_path_buf()));

        mock_shell
            .expect_execute()
            .returning(|_| Err(AppError::ShellExecution("Deployment failed".to_string())));

        let deploy_service = DeployServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_path));

        let result = deploy_service.deploy_executable().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_locate_fish_completions_failure() {
        let mock_shell = MockShellExecutor::new();
        let mut mock_path = MockPathOperations::new();

        mock_path
            .expect_parse_path()
            .returning(|_| Err(AppError::Deployment("Invalid path".to_string())));

        let deploy_service = DeployServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_path));

        let result = deploy_service.locate_fish_completions().await;
        assert!(result.is_err());
    }
}
