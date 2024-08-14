use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait PromptOperations: Send + Sync {
    async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
}
