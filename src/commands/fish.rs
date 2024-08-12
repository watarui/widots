use log::debug;

use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::{Argument, Cmd};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;

pub trait FishOperations {
    fn install(&self) -> Result<(), AppError>;
    fn set_default(&self) -> Result<(), AppError>;
}

pub struct Fish {
    shell: Arc<dyn ShellOperations>,
}

impl Fish {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    fn get_fish_path(&self) -> Result<Option<String>, AppError> {
        let output = self.shell.which(Cmd::Cmd("fish".into()))?;

        debug!("Get fish path output: {:?}", output);

        if output.status.success() {
            Ok(Some(
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
            ))
        } else {
            Ok(None)
        }
    }

    fn is_homebrew_installed(&self) -> Result<bool, AppError> {
        Ok(self.shell.which(Cmd::Cmd("brew".into())).is_ok())
    }

    fn is_default(&self, fish_path: &str) -> Result<bool, AppError> {
        let user_path = Path::new("/Users").join(whoami::username());
        let output = self.shell.shell(Cmd::Cmd(
            format!("dscl . -read {} UserShell", user_path.display()).into(),
        ))?;

        debug!("Confirm if fish is default output: {:?}", output);

        Ok(String::from_utf8_lossy(&output.stdout).contains(fish_path))
    }

    fn add_fish_to_etc_shells(&self, fish_path: &str) -> Result<(), AppError> {
        let shells = self
            .shell
            .sudo(Cmd::Cmd("cat".into()), Argument::Arg("/etc/shells".into()))?;
        let shells = String::from_utf8_lossy(&shells.stdout);

        debug!("Confirm /etc/shells output: {:?}", shells);

        if !shells.lines().any(|line| line == fish_path) {
            println!("Adding Fish to /etc/shells...");
            let mut child = Command::new("sudo")
                .arg("tee")
                .arg("-a")
                .arg("/etc/shells")
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| AppError::ExecuteCommandFailed(e.to_string()))?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(fish_path.as_bytes())
                    .map_err(|e| AppError::ExecuteCommandFailed(e.to_string()))?;
            }

            let status = child
                .wait()
                .map_err(|e| AppError::ExecuteCommandFailed(e.to_string()))?;

            if !status.success() {
                return Err(AppError::AddFishToEtcShellsFailed);
            }
        }
        Ok(())
    }

    fn change_default_shell(&self, fish_path: &str) -> Result<(), AppError> {
        println!("Changing default shell to Fish. You may be prompted for your password.");
        let output = self.shell.sudo(
            Cmd::Cmd("chsh".into()),
            Argument::Args(vec![
                "-s".into(),
                fish_path.into(),
                whoami::username().into(),
            ]),
        )?;

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

impl FishOperations for Fish {
    fn install(&self) -> Result<(), AppError> {
        if let Some(fish_path) = self.get_fish_path()? {
            println!("Fish shell is already installed. Path: {}", fish_path);
            return Ok(());
        }

        println!("Fish shell not found. Installing using Homebrew...");

        if !self.is_homebrew_installed()? {
            return Err(AppError::Homebrew("Homebrew is not installed".to_string()));
        }

        let output = self.shell.shell(Cmd::Cmd("brew install fish".into()))?;

        if output.status.success() {
            println!("Fish shell installed successfully");
            Ok(())
        } else {
            Err(AppError::Fish("Failed to install fish shell".to_string()))
        }
    }

    fn set_default(&self) -> Result<(), AppError> {
        let fish_path = self
            .get_fish_path()?
            .ok_or(AppError::Fish("Fish not found".to_string()))?;

        if self.is_default(&fish_path)? {
            println!("Fish is already the default shell");
            return Ok(());
        }

        self.add_fish_to_etc_shells(&fish_path)?;
        self.change_default_shell(&fish_path)
    }
}
