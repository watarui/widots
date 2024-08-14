use async_trait::async_trait;

use crate::error::AppError;

#[async_trait]
pub trait PromptOperations {
    async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
}
