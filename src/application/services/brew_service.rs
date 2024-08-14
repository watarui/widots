use crate::constants::{BREW_CASK_FORMULA_FILENAME, BREW_FORMULA_FILENAME, RESOURCES_DIR};
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait BrewService: Send + Sync {
    async fn install(&self) -> Result<(), AppError>;
    async fn import(&self) -> Result<(), AppError>;
    async fn export(&self) -> Result<(), AppError>;
}

pub struct BrewServiceImpl {
    shell_executor: Arc<dyn ShellExecutor>,
    fs_operations: Arc<dyn FileSystemOperations>,
}

impl BrewServiceImpl {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        fs_operations: Arc<dyn FileSystemOperations>,
    ) -> Self {
        Self {
            shell_executor,
            fs_operations,
        }
    }
}

#[async_trait]
impl BrewService for BrewServiceImpl {
    async fn install(&self) -> Result<(), AppError> {
        let install_script = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
        self.shell_executor.execute(install_script).await?;
        Ok(())
    }

    async fn import(&self) -> Result<(), AppError> {
        let import_path = Path::new(RESOURCES_DIR).join(BREW_FORMULA_FILENAME);
        let formulas = self.fs_operations.read_lines(import_path.as_path()).await?;
        for formula in formulas {
            self.shell_executor
                .execute(&format!("brew install {}", formula))
                .await?;
        }

        let import_path = Path::new(RESOURCES_DIR).join(BREW_CASK_FORMULA_FILENAME);
        let casks = self.fs_operations.read_lines(import_path.as_path()).await?;
        for cask in casks {
            self.shell_executor
                .execute(&format!("brew install --cask {}", cask))
                .await?;
        }

        Ok(())
    }

    async fn export(&self) -> Result<(), AppError> {
        let export_path = Path::new(RESOURCES_DIR).join(BREW_FORMULA_FILENAME);
        let formulas = self.shell_executor.execute("brew leaves").await?;
        self.fs_operations
            .write_lines(
                export_path.as_path(),
                &formulas
                    .lines()
                    .map(|formula| formula.to_string())
                    .collect::<Vec<_>>(),
            )
            .await?;

        let export_path = Path::new(RESOURCES_DIR).join(BREW_CASK_FORMULA_FILENAME);
        let casks = self.shell_executor.execute("brew list --cask").await?;
        self.fs_operations
            .write_lines(
                export_path.as_path(),
                &casks
                    .lines()
                    .map(|cask| cask.to_string())
                    .collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }
}
