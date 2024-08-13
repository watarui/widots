use crate::application::services::brew_service::BrewService;
use crate::application::services::deploy_service::DeployService;
use crate::application::services::fish_service::FishService;
use crate::application::services::link_service::LinkService;
use crate::application::services::vscode_service::VSCodeService;
use crate::config::Config;
use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::path::PathOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::deploy::Deployer;
use crate::infrastructure::fs::FileSystemOperationsImpl;
use crate::infrastructure::link::LinkerImpl;
use crate::infrastructure::os::OSDetector;
use crate::infrastructure::path::PathExpander;
use crate::infrastructure::shell::executor::SystemShellExecutor;
use crate::utils::yaml::YamlParser;
use std::sync::Arc;

pub struct AppConfig {
    pub _config: Config,
    pub shell_executor: Arc<dyn ShellExecutor>,
    pub os_detector: Arc<dyn OSOperations>,
    pub path_operations: Arc<dyn PathOperations>,
    pub link_service: LinkService,
    pub deploy_service: DeployService,
    pub brew_service: BrewService,
    pub fish_service: FishService,
    pub vscode_service: VSCodeService,
    pub yaml_parser: Arc<YamlParser>,
}

impl AppConfig {
    pub async fn new() -> Result<Self, AppError> {
        let config = Config::load().map_err(|e| AppError::ConfigError(e.to_string()))?;

        let shell_executor = Arc::new(SystemShellExecutor::new());
        let os_detector = Arc::new(OSDetector::new());
        let fs_operations = Arc::new(FileSystemOperationsImpl::new());
        let path_operations = Arc::new(PathExpander::new());
        let yaml_parser = Arc::new(YamlParser::new());
        let deploy_operations = Arc::new(Deployer::new());

        let link_operations: Arc<dyn LinkOperations> =
            Arc::new(LinkerImpl::new(path_operations.clone()));

        let link_service = LinkService::new(link_operations, path_operations.clone());

        let deploy_service = DeployService::new(deploy_operations.clone(), shell_executor.clone());
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
            path_operations,
            link_service,
            deploy_service,
            brew_service,
            fish_service,
            vscode_service,
            yaml_parser,
        })
    }

    pub fn _get_shell_executor(&self) -> Arc<dyn ShellExecutor> {
        self.shell_executor.clone()
    }

    pub fn _get_os_detector(&self) -> Arc<dyn OSOperations> {
        self.os_detector.clone()
    }

    pub fn _get_path_operations(&self) -> Arc<dyn PathOperations> {
        self.path_operations.clone()
    }

    pub fn get_link_service(&self) -> &LinkService {
        &self.link_service
    }

    pub fn get_deploy_service(&self) -> &DeployService {
        &self.deploy_service
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

    pub fn _get_yaml_parser(&self) -> Arc<YamlParser> {
        self.yaml_parser.clone()
    }
}
