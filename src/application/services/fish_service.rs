use crate::domain::os::OSOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait FishService: Send + Sync {
    async fn install(&self) -> Result<(), AppError>;
    async fn set_default(&self) -> Result<(), AppError>;
    async fn install_fisher(&self) -> Result<(), AppError>;
}

pub struct FishServiceImpl {
    shell_executor: Arc<dyn ShellExecutor>,
    os_detector: Arc<dyn OSOperations>,
}

impl FishServiceImpl {
    pub fn new(shell_executor: Arc<dyn ShellExecutor>, os_detector: Arc<dyn OSOperations>) -> Self {
        Self {
            shell_executor,
            os_detector,
        }
    }
}

#[async_trait]
impl FishService for FishServiceImpl {
    async fn install(&self) -> Result<(), AppError> {
        let os = self.os_detector.get_os().await?;
        match os.as_str() {
            "macos" => {
                self.shell_executor
                    .execute("brew", &["install", "fish"])
                    .await?
            }
            "linux" => {
                // This is a simplification. In reality, you'd need to handle different Linux distributions.
                self.shell_executor
                    .execute("sudo", &["apt-get", "install", "fish"])
                    .await?
            }
            _ => return Err(AppError::UnsupportedOS(os)),
        };
        Ok(())
    }

    async fn set_default(&self) -> Result<(), AppError> {
        let fish_path = self.shell_executor.execute("which", &["fish"]).await?;
        self.shell_executor
            .execute(
                "echo",
                &[
                    "'",
                    fish_path.trim(),
                    "'",
                    "|",
                    "sudo",
                    "tee",
                    "-a",
                    "/etc/shells",
                ],
            )
            .await?;
        self.shell_executor
            .execute("chsh", &["-s", fish_path.trim()])
            .await?;
        Ok(())
    }

    async fn install_fisher(&self) -> Result<(), AppError> {
        let install_script =
            r#"curl -sL https://git.io/fisher | source && fisher install jorgebucaran/fisher"#;
        self.shell_executor
            .execute("fish", &["-c", "'", install_script, "'"])
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::os::OSOperations;
    use crate::domain::shell::ShellExecutor;
    use crate::error::AppError;
    use async_trait::async_trait;
    use mockall::mock;
    use std::process::Output;
    use std::sync::Arc;

    mock! {
        ShellExecutor {}
        #[async_trait]
        impl ShellExecutor for ShellExecutor {
            async fn execute<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<String, AppError>;
            async fn output<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<Output, AppError>;
            fn stderr(&self, output: &Output) -> String;
        }
    }

    mock! {
        OSOperations {}
        #[async_trait]
        impl OSOperations for OSOperations {
            async fn get_os(&self) -> Result<String, AppError>;
        }
    }

    #[tokio::test]
    async fn test_fish_install() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_os = MockOSOperations::new();

        mock_os
            .expect_get_os()
            .returning(|| Ok("macos".to_string()));

        mock_shell
            .expect_execute()
            .withf(|cmd: &str, args: &[&str]| cmd == "brew" && args == ["install", "fish"])
            .returning(|_, _| Ok("Fish installed successfully".to_string()));

        let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

        let result = fish_service.install().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fish_set_default() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .returning(|_, _| Ok("Command executed successfully".to_string()));

        let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

        let result = fish_service.set_default().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fish_install_fisher() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .returning(|_, _| Ok("Fisher installed successfully".to_string()));

        let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

        let result = fish_service.install_fisher().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fish_install_unsupported_os() {
        let mock_shell = MockShellExecutor::new();
        let mut mock_os = MockOSOperations::new();

        mock_os
            .expect_get_os()
            .returning(|| Ok("windows".to_string()));

        let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

        let result = fish_service.install().await;
        assert!(matches!(result, Err(AppError::UnsupportedOS(_))));
    }

    #[tokio::test]
    async fn test_fish_set_default_failure() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .returning(|_, _| Err(AppError::ShellExecution("Command failed".to_string())));

        let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

        let result = fish_service.set_default().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fish_install_fisher_failure() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_os = MockOSOperations::new();

        mock_shell.expect_execute().returning(|_, _| {
            Err(AppError::ShellExecution(
                "Fisher installation failed".to_string(),
            ))
        });

        let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

        let result = fish_service.install_fisher().await;
        assert!(result.is_err());
    }
}
