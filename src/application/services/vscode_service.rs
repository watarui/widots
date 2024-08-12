use crate::config::PathConfig;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use std::sync::Arc;

pub struct VSCodeService {
    shell_executor: Arc<dyn ShellExecutor>,
    fs_operations: Arc<dyn FileSystemOperations>,
    path_config: PathConfig,
}

impl VSCodeService {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        fs_operations: Arc<dyn FileSystemOperations>,
        path_config: &PathConfig,
    ) -> Self {
        Self {
            shell_executor,
            fs_operations,
            path_config: path_config.clone(),
        }
    }

    pub async fn export_extensions(&self) -> Result<(), AppError> {
        let extensions = self
            .shell_executor
            .execute("code --list-extensions")
            .await?;
        let export_path = self.path_config.backups.join("vscode_extensions.txt");
        self.fs_operations
            .write_lines(
                &export_path,
                &extensions
                    .lines()
                    .map(|extension| extension.to_string())
                    .collect::<Vec<_>>(),
            )
            .await?;
        Ok(())
    }

    pub async fn import_extensions(&self) -> Result<(), AppError> {
        let import_path = self.path_config.backups.join("vscode_extensions.txt");
        let extensions = self.fs_operations.read_lines(&import_path).await?;
        for extension in extensions {
            self.shell_executor
                .execute(&format!("code --install-extension {}", extension))
                .await?;
        }
        Ok(())
    }

    pub async fn ensure_code_command(&self) -> Result<(), AppError> {
        match self.shell_executor.execute("which code").await {
            Ok(_) => Ok(()),
            Err(_) => {
                self.shell_executor.execute(
                    r#"ln -s "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" /usr/local/bin/code"#
                ).await?;
                Ok(())
            }
        }
    }
}
