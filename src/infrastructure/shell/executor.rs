use std::process::Output;

use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
use tokio::process::Command;

pub struct SystemShellExecutor;

impl Default for SystemShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ShellExecutor for SystemShellExecutor {
    async fn execute(&self, command: &str) -> Result<String, AppError> {
        let output = Command::new("sh").arg("-c").arg(command).output().await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AppError::ShellExecution(
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
            .map_err(AppError::Io)
    }

    fn stderr(&self, output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::shell::ShellExecutor;
    use crate::error::AppError;

    #[tokio::test]
    async fn test_execute() -> Result<(), AppError> {
        let shell_executor = SystemShellExecutor::new();
        let result = shell_executor.execute("echo 'Hello, World!'").await?;
        assert_eq!(result.trim(), "Hello, World!");
        Ok(())
    }

    #[tokio::test]
    async fn test_output() -> Result<(), AppError> {
        let shell_executor = SystemShellExecutor::new();
        let result = shell_executor.output("echo 'Hello, World!'").await?;
        assert!(result.status.success());
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim(),
            "Hello, World!"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_stderr() -> Result<(), AppError> {
        let shell_executor = SystemShellExecutor::new();
        let result = shell_executor.output("echo 'Error message' >&2").await?;
        let stderr = shell_executor.stderr(&result);
        assert_eq!(stderr.trim(), "Error message");
        Ok(())
    }
}
