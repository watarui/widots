use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::shell::executor::SystemShellExecutor;

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
