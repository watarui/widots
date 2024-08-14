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
