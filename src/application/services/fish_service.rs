use crate::domain::os::OSOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use std::sync::Arc;

pub struct FishService {
    shell_executor: Arc<dyn ShellExecutor>,
    // todo implement
    _fs_operations: Arc<dyn FileSystemOperations>,
    os_detector: Arc<dyn OSOperations>,
}

impl FishService {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        fs_operations: Arc<dyn FileSystemOperations>,
        os_detector: Arc<dyn OSOperations>,
    ) -> Self {
        Self {
            shell_executor,
            _fs_operations: fs_operations,
            os_detector,
        }
    }

    pub async fn install(&self) -> Result<(), AppError> {
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

    pub async fn set_default(&self) -> Result<(), AppError> {
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

    pub async fn install_fisher(&self) -> Result<(), AppError> {
        let install_script =
            r#"curl -sL https://git.io/fisher | source && fisher install jorgebucaran/fisher"#;
        self.shell_executor
            .execute(&format!("fish -c '{}'", install_script))
            .await?;
        Ok(())
    }
}
