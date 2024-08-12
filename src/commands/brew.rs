use async_trait::async_trait;
use log::debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::config::constants::{
    APP_RESOURCE_DIR, BREW_CASK_FORMULA_FILENAME, BREW_FORMULA_FILENAME,
};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::Cmd;

#[async_trait]
pub trait HomebrewOperations: Send + Sync {
    async fn install(&self) -> Result<(), AppError>;
    async fn import(&self) -> Result<(), AppError>;
    async fn export(&self) -> Result<(), AppError>;
    async fn is_installed(&self) -> bool;
}

pub struct Homebrew {
    shell: Arc<dyn ShellOperations>,
}

impl Homebrew {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    async fn install_homebrew(&self) -> Result<(), AppError> {
        let install_script = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
        let output = self.shell.bash(install_script).await?;

        debug!("Homebrew install output: {:?}", output);

        if output.status.success() {
            Ok(())
        } else {
            Err(AppError::Homebrew("Failed to install Homebrew".to_string()))
        }
    }

    async fn import_formula(&self, formula_path: &PathBuf) -> Result<(), AppError> {
        debug!("Importing formula... from: {:?}", formula_path.display());

        self.install_formula(formula_path, "").await
    }

    async fn import_cask(&self, cask_path: &PathBuf) -> Result<(), AppError> {
        debug!("Importing cask formula... from: {:?}", cask_path.display());

        self.install_formula(cask_path, "--cask").await
    }

    async fn install_formula(&self, formula_path: &PathBuf, arg: &str) -> Result<(), AppError> {
        debug!("Install formula path: {:?}", formula_path.display());

        let file = File::open(formula_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let formula = line.trim();
            if !formula.is_empty() {
                debug!("Install formula: {:?}", formula);

                let cmd = Cmd::new(format!("brew install {} {}", arg, formula));
                let output = self.shell.shell(&cmd).await?;

                debug!("Install formula output: {:?}", output);

                if !output.status.success() {
                    println!("Failed to install formula: {}", formula);
                }
            }
        }
        Ok(())
    }

    async fn export_formula(&self, args: &[String], path: &PathBuf) -> Result<(), AppError> {
        let cmd = Cmd::new(format!("brew {}", args.join(" ")));
        let output = self.shell.shell(&cmd).await?;

        debug!("Export formula output: {:?}", output);

        let mut file = File::create(path).await?;
        file.write_all(&output.stdout).await?;
        Ok(())
    }
}

#[async_trait]
impl HomebrewOperations for Homebrew {
    async fn install(&self) -> Result<(), AppError> {
        if self.is_installed().await {
            return Err(AppError::Homebrew(
                "Homebrew is already installed".to_string(),
            ));
        }

        self.install_homebrew().await
    }

    async fn import(&self) -> Result<(), AppError> {
        if !self.is_installed().await {
            return Err(AppError::Homebrew("Homebrew is not installed".to_string()));
        }

        let formula_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_FORMULA_FILENAME);
        let cask_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_CASK_FORMULA_FILENAME);

        self.import_formula(&formula_path).await?;
        self.import_cask(&cask_path).await?;

        Ok(())
    }

    async fn export(&self) -> Result<(), AppError> {
        if !self.is_installed().await {
            return Err(AppError::Homebrew("Homebrew is not installed".to_string()));
        }

        create_dir_all(APP_RESOURCE_DIR).await?;

        let formula_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_FORMULA_FILENAME);
        self.export_formula(&["leaves".to_string()], &formula_path)
            .await?;

        let cask_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_CASK_FORMULA_FILENAME);
        self.export_formula(
            &["list".to_string(), "--cask".to_string(), "-1".to_string()],
            &cask_path,
        )
        .await?;

        Ok(())
    }

    async fn is_installed(&self) -> bool {
        let cmd = Cmd::new("brew");
        self.shell.which(&cmd).await.is_ok()
    }
}
