use crate::application::services::brew_service::BrewService;
use crate::application::services::deploy_service::DeployService;
use crate::application::services::fish_service::FishService;
use crate::application::services::link_service::LinkService;
use crate::application::services::load_service::LoadService;
use crate::application::services::vscode_service::VSCodeService;
use crate::domain::link::LinkOperations;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperationsImpl;
use crate::infrastructure::link::LinkerImpl;
use crate::infrastructure::os::OSDetector;
use crate::infrastructure::path::PathExpander;
use crate::infrastructure::prompt::Prompt;
use crate::infrastructure::shell::executor::SystemShellExecutor;
use crate::utils::yaml::YamlParser;
use std::sync::Arc;

pub struct AppConfig {
    pub link_service: LinkService,
    pub load_service: LoadService,
    pub deploy_service: DeployService,
    pub brew_service: BrewService,
    pub fish_service: FishService,
    pub vscode_service: VSCodeService,
}

impl AppConfig {
    pub async fn new() -> Result<Self, AppError> {
        let shell_executor = Arc::new(SystemShellExecutor::new());
        let os_detector = Arc::new(OSDetector::new());
        let fs_operations = Arc::new(FileSystemOperationsImpl::new());
        let path_operations = Arc::new(PathExpander::new());
        let yaml_parser = Arc::new(YamlParser::new());
        let prompter = Arc::new(Prompt::new());

        let link_operations: Arc<dyn LinkOperations> =
            Arc::new(LinkerImpl::new(path_operations.clone()));

        let link_service = LinkService::new(
            link_operations.clone(),
            path_operations.clone(),
            prompter.clone(),
        );

        let load_service = LoadService::new(
            link_operations.clone(),
            path_operations.clone(),
            yaml_parser.clone(),
            os_detector.clone(),
            shell_executor.clone(),
            prompter.clone(),
        );

        let deploy_service = DeployService::new(shell_executor.clone(), path_operations.clone());
        let brew_service = BrewService::new(shell_executor.clone(), fs_operations.clone());

        let fish_service = FishService::new(
            shell_executor.clone(),
            fs_operations.clone(),
            os_detector.clone(),
        );

        let vscode_service = VSCodeService::new(shell_executor.clone(), fs_operations.clone());

        Ok(Self {
            link_service,
            load_service,
            deploy_service,
            brew_service,
            fish_service,
            vscode_service,
        })
    }

    pub fn get_link_service(&self) -> &LinkService {
        &self.link_service
    }

    pub fn get_load_service(&self) -> &LoadService {
        &self.load_service
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
}
