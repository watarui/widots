use crate::domain::os::OSOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
#[cfg(test)]
use mockall::mock;
#[cfg(test)]
use mockall::predicate::eq;
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
            "macos" => self.shell_executor.execute("brew install fish").await?,
            "linux" => {
                // This is a simplification. In reality, you'd need to handle different Linux distributions.
                self.shell_executor
                    .execute("sudo apt-get install fish")
                    .await?
            }
            _ => return Err(AppError::UnsupportedOS(os)),
        };
        Ok(())
    }

    async fn set_default(&self) -> Result<(), AppError> {
        let fish_path = self.shell_executor.execute("which fish").await?;
        self.shell_executor
            .execute(&format!(
                "echo '{}' | sudo tee -a /etc/shells",
                fish_path.trim()
            ))
            .await?;
        self.shell_executor
            .execute(&format!("chsh -s {}", fish_path.trim()))
            .await?;
        Ok(())
    }

    async fn install_fisher(&self) -> Result<(), AppError> {
        let install_script =
            r#"curl -sL https://git.io/fisher | source && fisher install jorgebucaran/fisher"#;
        self.shell_executor
            .execute(&format!("fish -c '{}'", install_script))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mock! {
    ShellExecutor {}
    #[async_trait]
    impl ShellExecutor for ShellExecutor {
        async fn execute(&self, command: &str) -> Result<String, AppError>;
        async fn output(&self, command: &str) -> Result<std::process::Output, AppError>;
        fn stderr(&self, output: &std::process::Output) -> String;
    }
}

#[cfg(test)]
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
        .with(eq("brew install fish"))
        .returning(|_| Ok("Fish installed successfully".to_string()));

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
        .returning(|_| Ok("Command executed successfully".to_string()));

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
        .returning(|_| Ok("Fisher installed successfully".to_string()));

    let fish_service = FishServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_os));

    let result = fish_service.install_fisher().await;
    assert!(result.is_ok());
}
