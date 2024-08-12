use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait ShellExecutor: Send + Sync {
    async fn execute(&self, command: &str) -> Result<String, AppError>;
}
