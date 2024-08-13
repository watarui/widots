use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait DeployOperations: Send + Sync {
    async fn build_project(&self) -> Result<(), AppError>;
    async fn deploy_executable(&self) -> Result<(), AppError>;
    async fn locate_fish_completions(&self) -> Result<(), AppError>;
}
