use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::models::config::Config;
use crate::models::link::FileProcessResult;
use crate::utils::toml::TomlOperations;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;

#[async_trait]
pub trait LoadService: Send + Sync {
    async fn load(&self, config_path: &Path, target: &Path, force: bool) -> Result<(), AppError>;
}

pub struct LoadServiceImpl {
    link_operations: Arc<dyn LinkOperations>,
    path_operations: Arc<dyn PathOperations>,
    toml_parser: Arc<dyn TomlOperations>,
    os_detector: Arc<dyn OSOperations>,
    shell_executor: Arc<dyn ShellExecutor>,
    prompter: Arc<dyn PromptOperations>,
}

impl LoadServiceImpl {
    pub fn new(
        link_operations: Arc<dyn LinkOperations>,
        path_operations: Arc<dyn PathOperations>,
        toml_parser: Arc<dyn TomlOperations>,
        os_detector: Arc<dyn OSOperations>,
        shell_executor: Arc<dyn ShellExecutor>,
        prompter: Arc<dyn PromptOperations>,
    ) -> Self {
        Self {
            link_operations,
            path_operations,
            toml_parser,
            os_detector,
            shell_executor,
            prompter,
        }
    }

    async fn evaluate_link_section(
        &self,
        config: &Config,
        target: &Path,
        force: bool,
    ) -> Result<(), AppError> {
        if let Some(links) = &config.link {
            for link in links {
                self.link_dotfiles(&link.location, target, force).await?;
            }
        }
        Ok(())
    }

    async fn evaluate_provision_section(&self, config: &Config) -> Result<(), AppError> {
        if let Some(provisions) = &config.provision {
            for provision in provisions {
                if provision.mode == self.os_detector.get_os().await? {
                    println!("🏃 Run provisioning... for {}", provision.mode);
                    self.run_bash_script(&provision.script).await?;
                    println!("🚀 Provisioning done");
                }
            }
        }
        Ok(())
    }

    async fn run_bash_script(&self, script: &str) -> Result<(), AppError> {
        let mut temp_file = NamedTempFile::new().map_err(|e| AppError::IoError(e.to_string()))?;
        temp_file
            .as_file_mut()
            .write_all(script.as_bytes())
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let command = format!("bash {}", temp_file.path().display());
        self.shell_executor.execute(&command).await?;
        Ok(())
    }

    async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let source = self.path_operations.parse_path(source).await?;
        let target = self.path_operations.parse_path(target).await?;

        let ans = self
            .prompter
            .confirm_action(&format!(
                "This will link files from {:?} to {:?}. Do you want to continue?",
                source.display(),
                target.display()
            ))
            .await?;
        if !ans {
            return Ok(vec![]);
        }

        self.link_operations
            .link_recursively(&source, &target, force)
            .await
    }
}

#[async_trait]
impl LoadService for LoadServiceImpl {
    async fn load(&self, config_path: &Path, target: &Path, force: bool) -> Result<(), AppError> {
        let config = self.toml_parser.parse(config_path).await?;

        self.evaluate_link_section(&config, target, force).await?;
        self.evaluate_provision_section(&config).await?;

        Ok(())
    }
}
