use crate::application::services::brew_service::BrewService;
use crate::application::services::fish_service::FishService;
use crate::application::services::link_service::LinkService;
use crate::application::services::vscode_service::VSCodeService;
use crate::config::Config;
use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperationsImpl;
use crate::infrastructure::os::OSDetector;
use crate::infrastructure::shell::executor::SystemShellExecutor;
use crate::utils::yaml::YamlParser;
use std::sync::Arc;

pub struct AppConfig {
    // todo implement
    pub _config: Config,
    pub shell_executor: Arc<dyn ShellExecutor>,
    pub os_detector: Arc<dyn OSOperations>,
    pub link_service: LinkService,
    pub brew_service: BrewService,
    pub fish_service: FishService,
    pub vscode_service: VSCodeService,
    pub yaml_parser: Arc<YamlParser>,
}

impl AppConfig {
    pub async fn new() -> Result<Self, AppError> {
        let config = Config::load().map_err(|e| AppError::ConfigError(e.to_string()))?;

        let shell_executor: Arc<dyn ShellExecutor> = Arc::new(SystemShellExecutor::new());
        let os_detector: Arc<dyn OSOperations> = Arc::new(OSDetector::new());
        let fs_operations = Arc::new(FileSystemOperationsImpl::new());
        let yaml_parser = Arc::new(YamlParser::new());

        let link_operations: Arc<dyn LinkOperations> =
            Arc::new(crate::infrastructure::link::LinkerImpl::new(
                fs_operations.clone(),
                shell_executor.clone(),
            ));

        let link_service = LinkService::new(link_operations);

        let brew_service =
            BrewService::new(shell_executor.clone(), fs_operations.clone(), &config.brew);

        let fish_service = FishService::new(
            shell_executor.clone(),
            fs_operations.clone(),
            os_detector.clone(),
        );

        let vscode_service =
            VSCodeService::new(shell_executor.clone(), fs_operations.clone(), &config.paths);

        Ok(Self {
            _config: config,
            shell_executor,
            os_detector,
            link_service,
            brew_service,
            fish_service,
            vscode_service,
            yaml_parser,
        })
    }

    pub fn get_shell_executor(&self) -> Arc<dyn ShellExecutor> {
        self.shell_executor.clone()
    }

    pub fn get_os_detector(&self) -> Arc<dyn OSOperations> {
        self.os_detector.clone()
    }

    pub fn get_link_service(&self) -> &LinkService {
        &self.link_service
    }

    pub fn get_brew_service(&self) -> &BrewService {
        &self.brew_service
    }

    pub fn get_fish_service(&self) -> &FishService {
        &self.fish_service
    }

    pub fn get_vscode_service(&self) -> &VSCodeService {
        &self.vscode_service
    }

    pub fn get_yaml_parser(&self) -> Arc<YamlParser> {
        self.yaml_parser.clone()
    }
}
