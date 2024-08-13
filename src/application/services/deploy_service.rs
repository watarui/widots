use crate::domain::deploy::DeployOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use std::sync::Arc;

pub struct DeployService {
    deploy_operations: Arc<dyn DeployOperations>,
    shell_executor: Arc<dyn ShellExecutor>,
}

impl DeployService {
    pub fn new(
        deploy_operations: Arc<dyn DeployOperations>,
        shell_executor: Arc<dyn ShellExecutor>,
    ) -> Self {
        Self {
            deploy_operations,
            shell_executor,
        }
    }

    // async fn deploy(&self) -> Result<(), AppError> {
    //     println!("Building the project in release mode...");
    //     if !self.build_project()? {
    //         return Err(AppError::Deployment("Build failed".to_string()));
    //     }

    //     println!("Deploying the executable...");
    //     self.deploy_executable()?;
    //     println!("Deployment successful!");

    //     println!("Locating fish shell command completion files...");
    //     self.locate_fish_completions()?;
    //     println!("Locate successful!");

    //     Ok(())
    // }

    // async fn build_release(&self) -> Result<(), AppError> {
    //     let output = self.shell_executor.execute("cargo build --release").await?;
    //     if output.status.success() {
    //         Ok(())
    //     } else {
    //         Err(AppError::Deployment("Build failed".to_string()))
    //     }
    // }

    pub async fn build(&self) -> Result<(), AppError> {
        self.deploy_operations.build_project().await
    }

    pub async fn deploy(&self) -> Result<(), AppError> {
        self.deploy_operations.deploy_executable().await;
        self.deploy_operations.locate_fish_completions().await
    }
}
