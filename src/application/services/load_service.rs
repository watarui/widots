use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::models::config::Config;
use crate::models::link::FileProcessResult;
use crate::utils::yaml::YamlOperations;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;

pub struct LoadService {
    link_operations: Arc<dyn LinkOperations>,
    path_operations: Arc<dyn PathOperations>,
    yaml_parser: Arc<dyn YamlOperations>,
    os_detector: Arc<dyn OSOperations>,
    shell_executor: Arc<dyn ShellExecutor>,
    prompter: Arc<dyn PromptOperations>,
}

impl LoadService {
    pub fn new(
        link_operations: Arc<dyn LinkOperations>,
        path_operations: Arc<dyn PathOperations>,
        yaml_parser: Arc<dyn YamlOperations>,
        os_detector: Arc<dyn OSOperations>,
        shell_executor: Arc<dyn ShellExecutor>,
        prompter: Arc<dyn PromptOperations>,
    ) -> Self {
        Self {
            link_operations,
            path_operations,
            yaml_parser,
            os_detector,
            shell_executor,
            prompter,
        }
    }

    pub async fn load_yaml(
        &self,
        yaml_path: &Path,
        target: &Path,
        force: bool,
    ) -> Result<(), AppError> {
        let yaml_script = self.yaml_parser.parse(yaml_path).await?;

        self.evaluate_link_section(&yaml_script, target, force)
            .await?;
        self.evaluate_provision_section(&yaml_script).await?;

        Ok(())
    }

    async fn evaluate_link_section(
        &self,
        yaml_script: &Config,
        target: &Path,
        force: bool,
    ) -> Result<(), AppError> {
        if let Some(links) = &yaml_script.link {
            for link in links {
                self.link_dotfiles(&link.location, target, force).await?;
            }
        }
        Ok(())
    }

    async fn evaluate_provision_section(&self, yaml_script: &Config) -> Result<(), AppError> {
        if let Some(provisions) = &yaml_script.provision {
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
            .write_all(script.as_bytes())
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let command = format!("bash {}", temp_file.path().display());
        self.shell_executor.execute(&command).await?;
        Ok(())
    }

    pub async fn link_dotfiles(
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
