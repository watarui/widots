use crate::application::services::brew_service::BrewService;
use crate::application::services::brew_service::BrewServiceImpl;
use crate::application::services::deploy_service::DeployService;
use crate::application::services::deploy_service::DeployServiceImpl;
use crate::application::services::fish_service::FishService;
use crate::application::services::fish_service::FishServiceImpl;
use crate::application::services::link_service::LinkService;
use crate::application::services::link_service::LinkServiceImpl;
use crate::application::services::load_service::LoadService;
use crate::application::services::load_service::LoadServiceImpl;
use crate::application::services::vscode_service::VSCodeService;
use crate::application::services::vscode_service::VSCodeServiceImpl;
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

pub trait ServiceProvider: Send + Sync {
    fn link_service(&self) -> Arc<dyn LinkService>;
    fn load_service(&self) -> Arc<dyn LoadService>;
    fn deploy_service(&self) -> Arc<dyn DeployService>;
    fn brew_service(&self) -> Arc<dyn BrewService>;
    fn fish_service(&self) -> Arc<dyn FishService>;
    fn vscode_service(&self) -> Arc<dyn VSCodeService>;
}

pub struct ProductionServiceProvider {
    link_service: Arc<dyn LinkService>,
    load_service: Arc<dyn LoadService>,
    deploy_service: Arc<dyn DeployService>,
    brew_service: Arc<dyn BrewService>,
    fish_service: Arc<dyn FishService>,
    vscode_service: Arc<dyn VSCodeService>,
}

impl ProductionServiceProvider {
    pub async fn new() -> Result<Self, AppError> {
        let shell_executor: Arc<dyn ShellExecutor> = Arc::new(SystemShellExecutor::new());
        let os_detector: Arc<dyn OSOperations> = Arc::new(OSDetector::new());
        let fs_operations: Arc<dyn FileSystemOperations> =
            Arc::new(FileSystemOperationsImpl::new());
        let path_operations: Arc<dyn PathOperations> = Arc::new(PathExpander::new());
        let toml_parser: Arc<dyn TomlOperations> = Arc::new(TomlParser::new());
        let prompter: Arc<dyn PromptOperations> = Arc::new(Prompt::new(false)); // false for production
        let link_operations: Arc<dyn LinkOperations> = Arc::new(LinkerImpl::new());

        Ok(Self {
            link_service: Arc::new(LinkServiceImpl::new(
                link_operations.clone(),
                path_operations.clone(),
                prompter.clone(),
            )),
            load_service: Arc::new(LoadServiceImpl::new(
                link_operations.clone(),
                path_operations.clone(),
                toml_parser.clone(),
                os_detector.clone(),
                shell_executor.clone(),
                prompter.clone(),
            )),
            deploy_service: Arc::new(DeployServiceImpl::new(
                shell_executor.clone(),
                path_operations.clone(),
            )),
            brew_service: Arc::new(BrewServiceImpl::new(
                shell_executor.clone(),
                fs_operations.clone(),
            )),
            fish_service: Arc::new(FishServiceImpl::new(
                shell_executor.clone(),
                os_detector.clone(),
            )),
            vscode_service: Arc::new(VSCodeServiceImpl::new(
                shell_executor.clone(),
                fs_operations.clone(),
            )),
        })
    }
}

impl ServiceProvider for ProductionServiceProvider {
    fn link_service(&self) -> Arc<dyn LinkService> {
        self.link_service.clone()
    }

    fn load_service(&self) -> Arc<dyn LoadService> {
        self.load_service.clone()
    }

    fn deploy_service(&self) -> Arc<dyn DeployService> {
        self.deploy_service.clone()
    }

    fn brew_service(&self) -> Arc<dyn BrewService> {
        self.brew_service.clone()
    }

    fn fish_service(&self) -> Arc<dyn FishService> {
        self.fish_service.clone()
    }

    fn vscode_service(&self) -> Arc<dyn VSCodeService> {
        self.vscode_service.clone()
    }
}

#[cfg(test)]
pub struct TestServiceProvider {
    link_service: Arc<dyn LinkService>,
    load_service: Arc<dyn LoadService>,
    deploy_service: Arc<dyn DeployService>,
    brew_service: Arc<dyn BrewService>,
    fish_service: Arc<dyn FishService>,
    vscode_service: Arc<dyn VSCodeService>,
}

#[cfg(test)]
impl TestServiceProvider {
    pub fn new(force: bool) -> Self {
        let shell_executor: Arc<dyn ShellExecutor> = Arc::new(SystemShellExecutor::new());
        let os_detector: Arc<dyn OSOperations> = Arc::new(OSDetector::new());
        let fs_operations: Arc<dyn FileSystemOperations> =
            Arc::new(FileSystemOperationsImpl::new());
        let path_operations: Arc<dyn PathOperations> = Arc::new(PathExpander::new());
        let toml_parser: Arc<dyn TomlOperations> = Arc::new(TomlParser::new());
        let prompter: Arc<dyn PromptOperations> = Arc::new(Prompt::new(force));
        let link_operations: Arc<dyn LinkOperations> = Arc::new(LinkerImpl::new());

        Self {
            link_service: Arc::new(LinkServiceImpl::new(
                link_operations.clone(),
                path_operations.clone(),
                prompter.clone(),
            )),
            load_service: Arc::new(LoadServiceImpl::new(
                link_operations.clone(),
                path_operations.clone(),
                toml_parser.clone(),
                os_detector.clone(),
                shell_executor.clone(),
                prompter.clone(),
            )),
            deploy_service: Arc::new(DeployServiceImpl::new(
                shell_executor.clone(),
                path_operations.clone(),
            )),
            brew_service: Arc::new(BrewServiceImpl::new(
                shell_executor.clone(),
                fs_operations.clone(),
            )),
            fish_service: Arc::new(FishServiceImpl::new(
                shell_executor.clone(),
                os_detector.clone(),
            )),
            vscode_service: Arc::new(VSCodeServiceImpl::new(
                shell_executor.clone(),
                fs_operations.clone(),
            )),
        }
    }
}

#[cfg(test)]
impl ServiceProvider for TestServiceProvider {
    fn link_service(&self) -> Arc<dyn LinkService> {
        self.link_service.clone()
    }

    fn load_service(&self) -> Arc<dyn LoadService> {
        self.load_service.clone()
    }

    fn deploy_service(&self) -> Arc<dyn DeployService> {
        self.deploy_service.clone()
    }

    fn brew_service(&self) -> Arc<dyn BrewService> {
        self.brew_service.clone()
    }

    fn fish_service(&self) -> Arc<dyn FishService> {
        self.fish_service.clone()
    }

    fn vscode_service(&self) -> Arc<dyn VSCodeService> {
        self.vscode_service.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_production_service_provider_creation() {
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async { ProductionServiceProvider::new().await });
        assert!(result.is_ok());
    }

    #[test]
    fn test_production_service_provider_services() {
        let rt = Runtime::new().unwrap();
        let provider = rt.block_on(async { ProductionServiceProvider::new().await.unwrap() });

        // Ensure that the services are not null
        assert!(Arc::strong_count(&provider.link_service()) > 0);
        assert!(Arc::strong_count(&provider.load_service()) > 0);
        assert!(Arc::strong_count(&provider.deploy_service()) > 0);
        assert!(Arc::strong_count(&provider.brew_service()) > 0);
        assert!(Arc::strong_count(&provider.fish_service()) > 0);
        assert!(Arc::strong_count(&provider.vscode_service()) > 0);
    }

    #[test]
    fn test_test_service_provider_creation() {
        let provider = TestServiceProvider::new(false);
        assert!(Arc::strong_count(&provider.link_service()) > 0);
        assert!(Arc::strong_count(&provider.load_service()) > 0);
        assert!(Arc::strong_count(&provider.deploy_service()) > 0);
        assert!(Arc::strong_count(&provider.brew_service()) > 0);
        assert!(Arc::strong_count(&provider.fish_service()) > 0);
        assert!(Arc::strong_count(&provider.vscode_service()) > 0);
    }

    #[test]
    fn test_test_service_provider_with_force() {
        let provider = TestServiceProvider::new(true);
        assert!(Arc::strong_count(&provider.link_service()) > 0);
        assert!(Arc::strong_count(&provider.load_service()) > 0);
        assert!(Arc::strong_count(&provider.deploy_service()) > 0);
        assert!(Arc::strong_count(&provider.brew_service()) > 0);
        assert!(Arc::strong_count(&provider.fish_service()) > 0);
        assert!(Arc::strong_count(&provider.vscode_service()) > 0);
    }
}
