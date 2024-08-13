use async_trait::async_trait;

use crate::{domain::deploy::DeployOperations, error::AppError};

pub struct Deployer;

impl Deployer {
    pub fn new() -> Self {
        Deployer
    }
}

#[async_trait]
impl DeployOperations for Deployer {
    async fn build_project(&self) -> Result<(), AppError> {
        // let output = self.shell_executor.execute("cargo build --release").await?;
        // Ok(output.status.success())
        Ok(())
    }
    async fn deploy_executable(&self) -> Result<(), AppError> {
        Ok(())
    }
    async fn locate_fish_completions(&self) -> Result<(), AppError> {
        Ok(())
    }
}
