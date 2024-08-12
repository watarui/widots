use crate::commands::link::LinkOperations;
use crate::core::os::OSOperations;
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::yaml::Yaml;
use crate::utils::yaml::YamlOperations;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait RunnerOperations: Send + Sync {
    async fn execute(&self, yaml_path: &Path, force: bool) -> Result<(), AppError>;
}

pub struct Runner {
    shell: Arc<dyn ShellOperations>,
    yaml_parser: Arc<dyn YamlOperations>,
    linker: Arc<dyn LinkOperations>,
    os_detector: Arc<dyn OSOperations>,
}

impl Runner {
    pub fn new(
        shell: Arc<dyn ShellOperations>,
        yaml_parser: Arc<dyn YamlOperations>,
        linker: Arc<dyn LinkOperations>,
        os_detector: Arc<dyn OSOperations>,
    ) -> Self {
        Self {
            shell,
            yaml_parser,
            linker,
            os_detector,
        }
    }

    async fn handle_links(&self, yaml_script: &Yaml, force: bool) -> Result<(), AppError> {
        if let Some(links) = &yaml_script.link {
            for link in links {
                self.linker
                    .link_recursively(&link.location, Path::new("~"), force)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_provision(&self, yaml_script: &Yaml) -> Result<(), AppError> {
        if let Some(provisions) = &yaml_script.provision {
            for provision in provisions {
                if provision.mode == self.os_detector.get_os().await? {
                    println!("🏃 Run provisioning... for {}", provision.mode);
                    self.shell.bash(&provision.script).await?;
                    println!("🚀 Provisioning done");
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl RunnerOperations for Runner {
    async fn execute(&self, yaml_path: &Path, force: bool) -> Result<(), AppError> {
        self.yaml_parser.validate_filename(yaml_path).await?;

        let yaml_script = self.yaml_parser.parse(yaml_path).await?;

        self.handle_links(&yaml_script, force).await?;
        self.handle_provision(&yaml_script).await?;

        Ok(())
    }
}
