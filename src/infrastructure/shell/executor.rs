use std::process::Output;

use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
use tokio::process::Command;

pub struct SystemShellExecutor;

impl SystemShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ShellExecutor for SystemShellExecutor {
    async fn execute(&self, command: &str) -> Result<String, AppError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| AppError::ShellExecutionError(e.to_string()))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AppError::ShellExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    async fn output(&self, command: &str) -> Result<Output, AppError> {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| AppError::ShellExecutionError(e.to_string()))
    }

    fn stderr(&self, output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}
