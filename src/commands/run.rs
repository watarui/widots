use crate::commands::link::LinkOperations;
use crate::core::os::{OSDetector, OSOperations};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::yaml::Yaml;
use crate::utils::yaml::YamlOperations;
use std::path::Path;
use std::sync::Arc;

pub trait RunnerOperations {
    fn execute(&self, yaml_path: &Path, force: bool) -> Result<(), AppError>;
}

pub struct Runner {
    shell: Arc<dyn ShellOperations>,
    yaml_parser: Arc<dyn YamlOperations>,
    linker: Arc<dyn LinkOperations>,
}

impl Runner {
    pub fn new(
        shell: Arc<dyn ShellOperations>,
        yaml_parser: Arc<dyn YamlOperations>,
        linker: Arc<dyn LinkOperations>,
    ) -> Self {
        Self {
            shell,
            yaml_parser,
            linker,
        }
    }

    fn handle_links(&self, yaml_script: &Yaml, force: bool) -> Result<(), AppError> {
        if let Some(links) = &yaml_script.link {
            for link in links {
                self.linker
                    .link_recursively(&link.location, Path::new("~"), force)?;
            }
        }
        Ok(())
    }

    fn handle_provision(&self, yaml_script: &Yaml) -> Result<(), AppError> {
        if let Some(provisions) = &yaml_script.provision {
            for provision in provisions {
                if provision.mode == OSDetector::new().get_os()? {
                    println!("🏃 Run provisioning... for {}", provision.mode);
                    self.shell.bash(&provision.script)?;
                    println!("🚀 Provisioning done");
                }
            }
        }
        Ok(())
    }
}

impl RunnerOperations for Runner {
    fn execute(&self, yaml_path: &Path, force: bool) -> Result<(), AppError> {
        self.yaml_parser.validate_filename(yaml_path)?;

        let yaml_script: Yaml = self.yaml_parser.parse(yaml_path)?;

        self.handle_links(&yaml_script, force)?;
        self.handle_provision(&yaml_script)?;

        Ok(())
    }
}
