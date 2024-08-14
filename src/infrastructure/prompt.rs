use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use async_trait::async_trait;
use inquire::Confirm;

pub struct Prompt;

impl Prompt {
    pub fn new() -> Self {
        Prompt
    }
}

#[async_trait]
impl PromptOperations for Prompt {
    async fn confirm_action(&self, message: &str) -> Result<bool, AppError> {
        Confirm::new(message)
            .with_default(false)
            .prompt()
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}
