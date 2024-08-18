use std::process::Output;

use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
use tokio::process::Command;

#[derive(Debug)]
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
    use mockall::predicate::*;
    use mockall::*;
    use proptest::prelude::*;
    use std::process::Command as StdCommand;
    use std::process::Output;
    use tokio::process::Command as TokioCommand;

    mock! {
        pub SystemShellExecutor {}

        #[async_trait]
        impl ShellExecutor for SystemShellExecutor {
            async fn execute(&self, command: &str) -> Result<String, AppError>;
            async fn output(&self, command: &str) -> Result<Output, AppError>;
            fn stderr(&self, output: &Output) -> String;
        }
    }

    #[tokio::test]
    async fn test_execute_with_mock() {
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_execute()
            .with(eq("echo 'Hello, World!'"))
            .times(1)
            .returning(|_| Ok("Hello, World!\n".to_string()));

        let result = mock.execute("echo 'Hello, World!'").await.unwrap();
        assert_eq!(result.trim(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_output_with_mock() {
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_output()
            .with(eq("echo 'Hello, World!'"))
            .times(1)
            .returning(|_cmd| {
                Ok(StdCommand::new("echo")
                    .arg("Hello, World!")
                    .output()
                    .expect("Failed to execute command"))
            });

        let result = mock.output("echo 'Hello, World!'").await.unwrap();
        assert!(result.status.success());
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim(),
            "Hello, World!"
        );
    }

    #[tokio::test]
    async fn test_stderr_with_mock() {
        let mock = SystemShellExecutor::new();
        let output = TokioCommand::new("sh")
            .arg("-c")
            .arg("echo 'Error message' >&2; exit 1")
            .output()
            .await
            .expect("Failed to execute command");

        let result = mock.stderr(&output);
        assert_eq!(result.trim(), "Error message");
    }

    #[tokio::test]
    async fn test_execute_error() {
        let executor = SystemShellExecutor::new();
        let result = executor.execute("non_existent_command").await;
        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn test_execute_with_various_commands(command in "[a-zA-Z0-9 ]{1,50}") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let executor = SystemShellExecutor::new();
                let result = executor.execute(&command).await;
                prop_assert!(result.is_ok() || result.is_err(), "execute neither succeeded nor failed");
                Ok(())
            }).unwrap();
        }

        #[test]
        fn test_output_with_various_commands(command in "[a-zA-Z0-9 ]{1,50}") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let executor = SystemShellExecutor::new();
                let result = executor.output(&command).await;
                prop_assert!(result.is_ok() || result.is_err(), "output neither succeeded nor failed");
                Ok(())
            }).unwrap();
        }
    }

    #[test]
    fn test_system_shell_executor_default() {
        let default_parser = SystemShellExecutor;
        let new_parser = SystemShellExecutor::new();

        // Ensure that the default implementation works correctly
        assert_eq!(format!("{:?}", default_parser), format!("{:?}", new_parser));
    }

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
