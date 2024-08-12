use async_trait::async_trait;
use log::debug;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::config::constants::{APP_RESOURCE_DIR, VSCODE_EXTENSIONS_FILENAME};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::Cmd;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[async_trait]
pub trait VSCodeOperations: Send + Sync {
    async fn export(&self) -> Result<(), AppError>;
    async fn import(&self) -> Result<(), AppError>;
    async fn ensure_code_command(&self) -> Result<(), AppError>;
}

pub struct VSCode {
    shell: Arc<dyn ShellOperations>,
}

impl VSCode {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    async fn check_code_command(&self) -> Result<(), AppError> {
        let cmd = Cmd::new("code");
        self.shell.which(&cmd).await?;
        Ok(())
    }

    async fn list_extensions(&self) -> Result<Vec<u8>, AppError> {
        let cmd = Cmd::new("code --list-extensions");
        let output = self.shell.shell(&cmd).await?;

        debug!("code --list-extensions output: {:?}", output);

        if !output.status.success() {
            return Err(AppError::VSCode("Failed to list extensions".to_string()));
        }
        Ok(output.stdout)
    }

    async fn save_extensions(&self, extensions: &[u8]) -> Result<(), AppError> {
        let path = self.get_extensions_path();
        create_dir_all(path.parent().unwrap()).await?;
        let mut file = File::create(&path).await?;
        file.write_all(extensions).await?;
        Ok(())
    }

    fn get_extensions_path(&self) -> PathBuf {
        Path::new(APP_RESOURCE_DIR).join(VSCODE_EXTENSIONS_FILENAME)
    }

    async fn read_extensions(&self, path: &Path) -> Result<Vec<String>, AppError> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut extensions = Vec::new();
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            extensions.push(line);
        }

        Ok(extensions)
    }

    async fn install_extension(&self, extension: &str) -> Result<(), AppError> {
        let cmd = Cmd::new(format!("code --install-extension --force {}", extension));
        let output = self.shell.shell(&cmd).await?;

        debug!("code --install-extension --force output: {:?}", output);

        if !output.status.success() {
            return Err(AppError::VSCode(format!(
                "Failed to install extension: {}",
                extension
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl VSCodeOperations for VSCode {
    async fn export(&self) -> Result<(), AppError> {
        self.check_code_command().await?;
        let extensions = self.list_extensions().await?;
        self.save_extensions(&extensions).await
    }

    async fn import(&self) -> Result<(), AppError> {
        let path = self.get_extensions_path();
        let extensions = self.read_extensions(&path).await?;
        for ext in extensions {
            self.install_extension(&ext).await?;
        }
        Ok(())
    }

    async fn ensure_code_command(&self) -> Result<(), AppError> {
        self.check_code_command().await?;
        println!("Code command is already installed");
        Ok(())
    }
}
