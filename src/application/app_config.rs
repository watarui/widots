use crate::application::services::brew_service::BrewService;
use crate::application::services::deploy_service::DeployService;
use crate::application::services::fish_service::FishService;
use crate::application::services::link_service::LinkService;
use crate::application::services::load_service::LoadService;
use crate::application::services::vscode_service::VSCodeService;
use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::{FileSystemOperations, FileSystemOperationsImpl};
use crate::infrastructure::link::LinkerImpl;
use crate::infrastructure::os::OSDetector;
use crate::infrastructure::path::PathExpander;
use crate::infrastructure::prompt::Prompt;
use crate::infrastructure::shell::executor::SystemShellExecutor;
use crate::utils::toml::{TomlOperations, TomlParser};
use std::sync::Arc;

use super::services::brew_service::BrewServiceImpl;
use super::services::deploy_service::DeployServiceImpl;
use super::services::fish_service::FishServiceImpl;
use super::services::link_service::LinkServiceImpl;
use super::services::load_service::LoadServiceImpl;
use super::services::vscode_service::VSCodeServiceImpl;

pub struct AppConfig {
    link_service: Arc<dyn LinkService>,
    load_service: Arc<dyn LoadService>,
    deploy_service: Arc<dyn DeployService>,
    brew_service: Arc<dyn BrewService>,
    fish_service: Arc<dyn FishService>,
    vscode_service: Arc<dyn VSCodeService>,
}

impl AppConfig {
    pub async fn new() -> Result<Self, AppError> {
        let shell_executor: Arc<dyn ShellExecutor> = Arc::new(SystemShellExecutor::new());
        let os_detector: Arc<dyn OSOperations> = Arc::new(OSDetector::new());
        let fs_operations: Arc<dyn FileSystemOperations> =
            Arc::new(FileSystemOperationsImpl::new());
        let path_operations: Arc<dyn PathOperations> = Arc::new(PathExpander::new());
        let toml_parser: Arc<dyn TomlOperations> = Arc::new(TomlParser::new());
        let prompter: Arc<dyn PromptOperations> = Arc::new(Prompt::new());

        let link_operations: Arc<dyn LinkOperations> =
            Arc::new(LinkerImpl::new(path_operations.clone()));

        let link_service = Arc::new(LinkServiceImpl::new(
            link_operations.clone(),
            path_operations.clone(),
            prompter.clone(),
        ));

        let load_service = Arc::new(LoadServiceImpl::new(
            link_operations.clone(),
            path_operations.clone(),
            toml_parser.clone(),
            os_detector.clone(),
            shell_executor.clone(),
            prompter.clone(),
        ));

        let deploy_service = Arc::new(DeployServiceImpl::new(
            shell_executor.clone(),
            path_operations.clone(),
        ));
        let brew_service = Arc::new(BrewServiceImpl::new(
            shell_executor.clone(),
            fs_operations.clone(),
        ));
        let fish_service = Arc::new(FishServiceImpl::new(
            shell_executor.clone(),
            fs_operations.clone(),
            os_detector.clone(),
        ));
        let vscode_service = Arc::new(VSCodeServiceImpl::new(
            shell_executor.clone(),
            fs_operations.clone(),
        ));

        Ok(Self {
            link_service,
            load_service,
            deploy_service,
            brew_service,
            fish_service,
            vscode_service,
        })
    }

    pub fn get_link_service(&self) -> &dyn LinkService {
        &*self.link_service
    }

    pub fn get_load_service(&self) -> &dyn LoadService {
        &*self.load_service
    }

    pub fn get_deploy_service(&self) -> &dyn DeployService {
        &*self.deploy_service
    }

    pub fn get_brew_service(&self) -> &dyn BrewService {
        &*self.brew_service
    }

    pub fn get_fish_service(&self) -> &dyn FishService {
        &*self.fish_service
    }

    pub fn get_vscode_service(&self) -> &dyn VSCodeService {
        &*self.vscode_service
    }
}
