use async_trait::async_trait;
use log::debug;
use std::collections::HashSet;
use std::sync::Arc;

use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::Cmd;

#[async_trait]
pub trait FisherOperations: Send + Sync {
    async fn install(&self) -> Result<(), AppError>;
}

pub struct Fisher {
    shell: Arc<dyn ShellOperations>,
}

impl Fisher {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    async fn is_installed(&self) -> bool {
        let cmd = Cmd::new("fish -c 'type -q fisher'");
        self.shell
            .shell(&cmd)
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    async fn install_fisher(&self) -> Result<std::process::Output, AppError> {
        let install_command = "curl -sL https://raw.githubusercontent.com/jorgebucaran/fisher/main/functions/fisher.fish | source && fisher install jorgebucaran/fisher";
        let cmd = Cmd::new(install_command);
        self.shell.shell(&cmd).await
    }

    async fn read_plugins(&self) -> Result<HashSet<String>, AppError> {
        let cmd = Cmd::new("fish -c 'fisher list'");
        let output = self.shell.shell(&cmd).await?;

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

    async fn install_plugins(&self, plugins: &HashSet<String>) -> Result<(), AppError> {
        for plugin in plugins {
            let cmd = Cmd::new(format!("fish -c 'fisher install {}'", plugin));
            let output = self.shell.shell(&cmd).await?;
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

#[async_trait]
impl FisherOperations for Fisher {
    async fn install(&self) -> Result<(), AppError> {
        if self.is_installed().await {
            return Err(AppError::Fish("Fisher is already installed".to_string()));
        }

        let output = self.install_fisher().await?;

        if output.status.success() {
            let plugins = self.read_plugins().await?;
            self.install_plugins(&plugins).await
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Fish(format!(
                "Failed to install Fisher: {}",
                error_message
            )))
        }
    }
}
