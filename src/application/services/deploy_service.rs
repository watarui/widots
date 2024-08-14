use async_trait::async_trait;
use tokio::fs;

use crate::config::constants::{
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
    path_expander: Arc<dyn PathOperations>,
}

impl DeployServiceImpl {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        path_expander: Arc<dyn PathOperations>,
    ) -> Self {
        Self {
            shell_executor,
            path_expander,
        }
    }

    async fn deploy_executable(&self) -> Result<(), AppError> {
        let source = Path::new(DEPLOY_SOURCE_PATH);
        let destination = self
            .path_expander
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
            .path_expander
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
