use crate::constants::{RESOURCES_DIR, VSCODE_EXTENSIONS_FILENAME};
use crate::domain::os::OSOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait VSCodeService: Send + Sync {
    async fn export_extensions(&self) -> Result<(), AppError>;
    async fn import_extensions(&self) -> Result<(), AppError>;
    async fn ensure_code_command(&self) -> Result<(), AppError>;
}

pub struct VSCodeServiceImpl {
    shell_executor: Arc<dyn ShellExecutor>,
    fs_operations: Arc<dyn FileSystemOperations>,
    os_detector: Arc<dyn OSOperations>,
}

impl VSCodeServiceImpl {
    pub fn new(
        shell_executor: Arc<dyn ShellExecutor>,
        fs_operations: Arc<dyn FileSystemOperations>,
        os_detector: Arc<dyn OSOperations>,
    ) -> Self {
        Self {
            shell_executor,
            fs_operations,
            os_detector,
        }
    }
}

#[async_trait]
impl VSCodeService for VSCodeServiceImpl {
    async fn export_extensions(&self) -> Result<(), AppError> {
        let extensions = self
            .shell_executor
            .execute("code --list-extensions")
            .await?;
        let export_path = Path::new(RESOURCES_DIR).join(VSCODE_EXTENSIONS_FILENAME);
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

    async fn import_extensions(&self) -> Result<(), AppError> {
        let import_path = Path::new(RESOURCES_DIR).join(VSCODE_EXTENSIONS_FILENAME);
        let extensions = self.fs_operations.read_lines(&import_path).await?;
        for extension in extensions {
            self.shell_executor
                .execute(&format!("code --install-extension {}", extension))
                .await?;
        }
        Ok(())
    }

    async fn ensure_code_command(&self) -> Result<(), AppError> {
        match self.shell_executor.execute("which code").await {
            Ok(_) => Ok(()),
            Err(_) => match self.os_detector.get_os().await?.as_str() {
                "macos" => {
                    if Path::new("/Applications/Visual Studio Code.app").exists() {
                        self.shell_executor.execute(
                    r#"ln -s "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" /usr/local/bin/code"#
                        ).await?;
                        println!("Code command installed successfully");
                        Ok(())
                    } else {
                        Err(AppError::CodeCommandNotInstalled)
                    }
                }
                _ => Err(AppError::CodeCommandNotInstalled),
            },
        }
    }
}

#[cfg(test)]
mod test {
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

    mock! {
        OSOperations {}
        #[async_trait]
        impl OSOperations for OSOperations {
            async fn get_os(&self) -> Result<String, AppError>;
        }
    }

    #[tokio::test]
    async fn test_export_extensions() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("code --list-extensions"))
            .returning(|_| Ok("extension1\nextension2".to_string()));

        mock_fs.expect_write_lines().returning(|_, _| Ok(()));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.export_extensions().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_extensions() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_fs
            .expect_read_lines()
            .returning(|_| Ok(vec!["extension1".to_string(), "extension2".to_string()]));

        mock_shell
            .expect_execute()
            .returning(|_| Ok("Extension installed successfully".to_string()));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.import_extensions().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ensure_code_command() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("which code"))
            .returning(|_| Ok("/usr/local/bin/code".to_string()));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.ensure_code_command().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_extensions_empty_list() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("code --list-extensions"))
            .returning(|_| Ok("".to_string()));

        mock_fs.expect_write_lines().returning(|_, lines| {
            assert!(lines.is_empty());
            Ok(())
        });

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.export_extensions().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_extensions_failure() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("code --list-extensions"))
            .returning(|_| Err(AppError::ShellExecution("Command failed".to_string())));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.export_extensions().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ShellExecution(_)));
    }

    #[tokio::test]
    async fn test_import_extensions_empty_file() {
        let mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_fs.expect_read_lines().returning(|_| Ok(vec![]));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.import_extensions().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_extensions_failure() {
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_fs
            .expect_read_lines()
            .returning(|_| Ok(vec!["extension1".to_string()]));

        mock_shell.expect_execute().returning(|_| {
            Err(AppError::ShellExecution(
                "Extension installation failed".to_string(),
            ))
        });

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.import_extensions().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ShellExecution(_)));
    }

    #[tokio::test]
    async fn test_ensure_code_command_already_exists() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();
        let mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("which code"))
            .returning(|_| Ok("/usr/local/bin/code".to_string()));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.ensure_code_command().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ensure_code_command_macos() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();
        let mut mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("which code"))
            .returning(|_| Err(AppError::ShellExecution("Command not found".to_string())));

        mock_os
            .expect_get_os()
            .returning(|| Ok("macos".to_string()));

        if Path::new("/Applications/Visual Studio Code.app").exists() {
            mock_shell
              .expect_execute()
              .with(eq(r#"ln -s "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" /usr/local/bin/code"#))
              .returning(|_| Ok("Symlink created".to_string()));
        }
        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.ensure_code_command().await;
        if Path::new("/Applications/Visual Studio Code.app").exists() {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                AppError::CodeCommandNotInstalled
            ));
        }
    }

    #[tokio::test]
    async fn test_ensure_code_command_non_macos() {
        let mut mock_shell = MockShellExecutor::new();
        let mock_fs = MockFileSystemOperations::new();
        let mut mock_os = MockOSOperations::new();

        mock_shell
            .expect_execute()
            .with(eq("which code"))
            .returning(|_| Err(AppError::ShellExecution("Command not found".to_string())));

        mock_os
            .expect_get_os()
            .returning(|| Ok("linux".to_string()));

        let vscode_service =
            VSCodeServiceImpl::new(Arc::new(mock_shell), Arc::new(mock_fs), Arc::new(mock_os));

        let result = vscode_service.ensure_code_command().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::CodeCommandNotInstalled
        ));
    }
}
