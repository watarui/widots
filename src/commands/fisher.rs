use log::debug;

use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::Cmd;
use std::collections::HashSet;
use std::sync::Arc;

pub trait FisherOperations {
    fn install(&self) -> Result<(), AppError>;
}

pub struct Fisher {
    shell: Arc<dyn ShellOperations>,
}

impl Fisher {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    fn is_installed(&self) -> bool {
        self.shell
            .shell(Cmd::Cmd("fish -c 'type -q fisher'".into()))
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn install_fisher(&self) -> Result<std::process::Output, AppError> {
        let install_command = "curl -sL https://raw.githubusercontent.com/jorgebucaran/fisher/main/functions/fisher.fish | source && fisher install jorgebucaran/fisher";
        self.shell.shell(Cmd::Cmd(install_command.into()))
    }

    fn read_plugins(&self) -> Result<HashSet<String>, AppError> {
        let output = self.shell.shell(Cmd::Cmd("fish -c 'fisher list'".into()))?;

        if !output.status.success() {
            return Err(AppError::Fish("Failed to list Fisher plugins".to_string()));
        }

        debug!("Fisher plugins: {:?}", output.stdout);

        let plugins = String::from_utf8(output.stdout)
            .map_err(|e| AppError::Unexpected(e.to_string()))?
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(plugins)
    }

    fn install_plugins(&self, plugins: &HashSet<String>) -> Result<(), AppError> {
        for plugin in plugins {
            let output = self.shell.shell(Cmd::Cmd(
                format!("fish -c 'fisher install {}'", plugin).into(),
            ))?;
            if !output.status.success() {
                let error_message = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Fish(format!(
                    "Failed to install Fisher plugin {}: {}",
                    plugin, error_message
                )));
            }
        }
        Ok(())
    }
}

impl FisherOperations for Fisher {
    fn install(&self) -> Result<(), AppError> {
        if self.is_installed() {
            return Err(AppError::Fish("Fisher is already installed".to_string()));
        }

        let output = self.install_fisher()?;

        if output.status.success() {
            let plugins = self.read_plugins()?;
            self.install_plugins(&plugins)
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Fish(format!(
                "Failed to install Fisher: {}",
                error_message
            )))
        }
    }
}
