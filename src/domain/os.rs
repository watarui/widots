use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait OSOperations: Send + Sync {
    async fn get_os(&self) -> Result<String, AppError>;
}
