use async_trait::async_trait;
use log::debug;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::{Argument, Cmd};

#[async_trait]
pub trait FishOperations: Send + Sync {
    async fn install(&self) -> Result<(), AppError>;
    async fn set_default(&self) -> Result<(), AppError>;
}

pub struct Fish {
    shell: Arc<dyn ShellOperations>,
}

impl Fish {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    async fn get_fish_path(&self) -> Result<Option<String>, AppError> {
        let cmd = Cmd::new("fish");
        let output = self.shell.which(&cmd).await?;

        debug!("Get fish path output: {:?}", output);

        if output.status.success() {
            Ok(Some(
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
            ))
        } else {
            Ok(None)
        }
    }

    async fn is_homebrew_installed(&self) -> Result<bool, AppError> {
        let cmd = Cmd::new("brew");
        Ok(self.shell.which(&cmd).await.is_ok())
    }

    async fn is_default(&self, fish_path: &str) -> Result<bool, AppError> {
        let user_path = Path::new("/Users").join(whoami::username());
        let cmd = Cmd::new(format!("dscl . -read {} UserShell", user_path.display()));
        let output = self.shell.shell(&cmd).await?;

        debug!("Confirm if fish is default output: {:?}", output);

        Ok(String::from_utf8_lossy(&output.stdout).contains(fish_path))
    }

    async fn add_fish_to_etc_shells(&self, fish_path: &str) -> Result<(), AppError> {
        let cmd = Cmd::new("cat");
        let arg = Argument::Arg("/etc/shells".into());
        let shells = self.shell.sudo(&cmd, &arg).await?;
        let shells = String::from_utf8_lossy(&shells.stdout);

        debug!("Confirm /etc/shells output: {:?}", shells);

        if !shells.lines().any(|line| line == fish_path) {
            println!("Adding Fish to /etc/shells...");
            let mut child = tokio::process::Command::new("sudo")
                .arg("tee")
                .arg("-a")
                .arg("/etc/shells")
                .stdin(Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(fish_path.as_bytes()).await?;
            }

            let status = child.wait().await?;

            if !status.success() {
                return Err(AppError::AddFishToEtcShellsFailed);
            }
        }
        Ok(())
    }

    async fn change_default_shell(&self, fish_path: &str) -> Result<(), AppError> {
        println!("Changing default shell to Fish. You may be prompted for your password.");
        let cmd = Cmd::new("chsh");
        let args = Argument::Args(vec![
            "-s".into(),
            fish_path.into(),
            whoami::username().into(),
        ]);
        let output = self.shell.sudo(&cmd, &args).await?;

        if output.status.success() {
            println!("Default shell has been changed to Fish. Please log out and log back in for the changes to take effect.");
            Ok(())
        } else {
            Err(AppError::Fish(
                "Failed to change default shell to Fish".to_string(),
            ))
        }
    }
}

#[async_trait]
impl FishOperations for Fish {
    async fn install(&self) -> Result<(), AppError> {
        if let Some(fish_path) = self.get_fish_path().await? {
            println!("Fish shell is already installed. Path: {}", fish_path);
            return Ok(());
        }

        println!("Fish shell not found. Installing using Homebrew...");

        if !self.is_homebrew_installed().await? {
            return Err(AppError::Homebrew("Homebrew is not installed".to_string()));
        }

        let cmd = Cmd::new("brew install fish");
        let output = self.shell.shell(&cmd).await?;

        if output.status.success() {
            println!("Fish shell installed successfully");
            Ok(())
        } else {
            Err(AppError::Fish("Failed to install fish shell".to_string()))
        }
    }

    async fn set_default(&self) -> Result<(), AppError> {
        let fish_path = self
            .get_fish_path()
            .await?
            .ok_or(AppError::Fish("Fish not found".to_string()))?;

        if self.is_default(&fish_path).await? {
            println!("Fish is already the default shell");
            return Ok(());
        }

        self.add_fish_to_etc_shells(&fish_path).await?;
        self.change_default_shell(&fish_path).await
    }
}
