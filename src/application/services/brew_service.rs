use crate::config::BrewConfig;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use std::sync::Arc;

pub struct BrewService {
    shell_executor: Arc<dyn ShellExecutor>,
    fs_operations: Arc<dyn FileSystemOperations>,
    config: BrewConfig,
}

impl BrewService {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        fs_operations: Arc<dyn FileSystemOperations>,
        config: &BrewConfig,
    ) -> Self {
        Self {
            shell_executor,
            fs_operations,
            config: config.clone(),
        }
    }

    pub async fn install(&self) -> Result<(), AppError> {
        let install_script = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
        self.shell_executor.execute(install_script).await?;
        Ok(())
    }

    pub async fn import(&self) -> Result<(), AppError> {
        let formulas = self
            .fs_operations
            .read_lines(&self.config.formula_file)
            .await?;
        for formula in formulas {
            self.shell_executor
                .execute(&format!("brew install {}", formula))
                .await?;
        }

        let casks = self
            .fs_operations
            .read_lines(&self.config.cask_file)
            .await?;
        for cask in casks {
            self.shell_executor
                .execute(&format!("brew install --cask {}", cask))
                .await?;
        }

        Ok(())
    }

    pub async fn export(&self) -> Result<(), AppError> {
        let formulas = self.shell_executor.execute("brew leaves").await?;
        self.fs_operations
            .write_lines(
                &self.config.formula_file,
                &formulas
                    .lines()
                    .map(|formula| formula.to_string())
                    .collect::<Vec<_>>(),
            )
            .await?;

        let casks = self.shell_executor.execute("brew list --cask").await?;
        self.fs_operations
            .write_lines(
                &self.config.cask_file,
                &casks
                    .lines()
                    .map(|cask| cask.to_string())
                    .collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }
}
