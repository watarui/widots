use crate::commands::brew::Homebrew;
use crate::commands::deploy::Deployer;
use crate::commands::fish::Fish;
use crate::commands::fisher::Fisher;
use crate::commands::link::Linker;
use crate::commands::materialize::Materializer;
use crate::commands::run::Runner;
use crate::commands::vscode::VSCode;
use crate::core::os::OSOperations;
use crate::core::path::PathOperations;
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::utils::yaml::YamlOperations;
use std::sync::Arc;

pub struct AppConfig {
    os_detector: Arc<dyn OSOperations>,
    path_expander: Arc<dyn PathOperations>,
    shell_executor: Arc<dyn ShellOperations>,
    yaml_parser: Arc<dyn YamlOperations>,
}

impl AppConfig {
    pub async fn new() -> Result<Self, AppError> {
        // Here you would typically load configuration from a file or environment variables
        // For simplicity, we're creating instances directly
        let os_detector = Arc::new(crate::core::os::OSDetector::new());
        let path_expander = Arc::new(crate::core::path::PathExpander::new());
        let shell_executor = Arc::new(crate::core::shell::ShellExecutor::new());
        let yaml_parser = Arc::new(crate::utils::yaml::YamlParser::new());

        Ok(Self {
            os_detector,
            path_expander,
            shell_executor,
            yaml_parser,
        })
    }

    pub fn homebrew(&self) -> Homebrew {
        Homebrew::new(self.shell_executor.clone())
    }

    pub fn deployer(&self) -> Deployer {
        Deployer::new(self.shell_executor.clone())
    }

    pub fn fish(&self) -> Fish {
        Fish::new(self.shell_executor.clone())
    }

    pub fn fisher(&self) -> Fisher {
        Fisher::new(self.shell_executor.clone())
    }

    pub fn linker(&self) -> Arc<dyn crate::commands::link::LinkOperations> {
        Linker::new(self.path_expander.clone())
    }

    pub fn materializer(&self) -> Materializer {
        Materializer::new(self.linker())
    }

    pub fn runner(&self) -> Runner {
        Runner::new(
            self.shell_executor.clone(),
            self.yaml_parser.clone(),
            self.linker(),
            self.os_detector.clone(),
        )
    }

    pub fn vscode(&self) -> VSCode {
        VSCode::new(self.shell_executor.clone())
    }
}
