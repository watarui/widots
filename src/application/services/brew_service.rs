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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shell::ShellExecutor;
    use crate::error::AppError;
    use crate::infrastructure::fs::FileSystemOperations;
    use async_trait::async_trait;
    use mockall::mock;
    use mockall::predicate::eq;
    use std::path::Path;
    use std::sync::Arc;

    mock! {
        ShellExecutor {}
        #[async_trait]
        impl ShellExecutor for ShellExecutor {
            async fn execute(&self, command: &str) -> Result<String, AppError>;
            async fn output(&self, command: &str) -> Result<std::process::Output, AppError>;
            fn stderr(&self, output: &std::process::Output) -> String;
        }
    }

    mock! {
        FileSystemOperations {}
        #[async_trait]
        impl FileSystemOperations for FileSystemOperations {
            async fn read_lines(&self, path: &Path) -> Result<Vec<String>, AppError>;
            async fn write_lines(&self, path: &Path, lines: &[String]) -> Result<(), AppError>;
        }
    }

    #[tokio::test]
    async fn test_brew_install() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();

        mock_shell
        .expect_execute()
        .with(eq("/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""))
        .returning(|_| Ok("Homebrew installed successfully".to_string()));

        let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

        let result = brew_service.install().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_brew_import() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();

        mock_fs
            .expect_read_lines()
            .returning(|_| Ok(vec!["package1".to_string(), "package2".to_string()]));

        mock_shell
            .expect_execute()
            .returning(|_| Ok("Package installed successfully".to_string()));

        let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

        let result = brew_service.import().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_brew_export() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();

        mock_shell
            .expect_execute()
            .returning(|_| Ok("package1\npackage2".to_string()));

        mock_fs.expect_write_lines().returning(|_, _| Ok(()));

        let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

        let result = brew_service.export().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_brew_install_failure() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();

        mock_shell
            .expect_execute()
            .returning(|_| Err(AppError::ShellExecution("Installation failed".to_string())));

        let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

        let result = brew_service.install().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_brew_import_empty_file() {
        let mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();

        mock_fs.expect_read_lines().returning(|_| Ok(vec![]));

        let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

        let result = brew_service.import().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_brew_export_empty_list() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();

        mock_shell
            .expect_execute()
            .returning(|_| Ok("".to_string()));
        mock_fs.expect_write_lines().returning(|_, _| Ok(()));

        let brew_service = BrewServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs));

        let result = brew_service.export().await;
        assert!(result.is_ok());
    }
}
