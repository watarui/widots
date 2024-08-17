use crate::application::service_provider::ServiceProvider;
use crate::error::AppError;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct FishArgs {
    #[clap(subcommand, value_enum, help = "Fish shell operation to execute")]
    command: FishCommands,
}

#[derive(Subcommand)]
enum FishCommands {
    Install,
    SetDefault,
    InstallFisher,
}

pub async fn execute(args: FishArgs, services: &dyn ServiceProvider) -> Result<(), AppError> {
    match args.command {
        FishCommands::Install => {
            services.fish_service().install().await?;
            println!("Fish shell installed successfully");
        }
        FishCommands::SetDefault => {
            services.fish_service().set_default().await?;
            println!("Fish shell set as default successfully");
        }
        FishCommands::InstallFisher => {
            services.fish_service().install_fisher().await?;
            println!("Fisher plugin manager installed successfully");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::application::service_provider::ServiceProvider;
    use crate::application::services::brew_service::BrewService;
    use crate::application::services::deploy_service::DeployService;
    use crate::application::services::fish_service::FishService;
    use crate::application::services::link_service::LinkService;
    use crate::application::services::load_service::LoadService;
    use crate::application::services::vscode_service::VSCodeService;
    use crate::error::AppError;
    use crate::models::link::FileProcessResult;
    use crate::presentation::cli::commands::fish::{execute, FishArgs, FishCommands};
    use async_trait::async_trait;
    use std::path::Path;
    use std::sync::Arc;

    struct CustomMockBrewService;

    #[async_trait]
    impl BrewService for CustomMockBrewService {
        async fn install(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn import(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn export(&self) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct CustomMockLinkService;

    #[async_trait]
    impl LinkService for CustomMockLinkService {
        async fn link_dotfiles(
            &self,
            _source: &Path,
            _target: &Path,
        ) -> Result<Vec<FileProcessResult>, AppError> {
            Ok(vec![])
        }

        async fn materialize_dotfiles(
            &self,
            _target: &Path,
        ) -> Result<Vec<FileProcessResult>, AppError> {
            Ok(vec![])
        }
    }

    struct CustomMockLoadService;

    #[async_trait]
    impl LoadService for CustomMockLoadService {
        async fn load(&self, _config_path: &Path, _target: &Path) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct CustomMockDeployService;

    #[async_trait]
    impl DeployService for CustomMockDeployService {
        async fn execute(&self) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct CustomMockFishService;

    #[async_trait]
    impl FishService for CustomMockFishService {
        async fn install(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn set_default(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn install_fisher(&self) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct CustomMockVSCodeService;

    #[async_trait]
    impl VSCodeService for CustomMockVSCodeService {
        async fn export_extensions(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn import_extensions(&self) -> Result<(), AppError> {
            Ok(())
        }

        async fn ensure_code_command(&self) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct CustomMockServiceProvider {
        brew_service: Arc<dyn BrewService>,
        link_service: Arc<dyn LinkService>,
        load_service: Arc<dyn LoadService>,
        deploy_service: Arc<dyn DeployService>,
        fish_service: Arc<dyn FishService>,
        vscode_service: Arc<dyn VSCodeService>,
    }

    impl CustomMockServiceProvider {
        fn new() -> Self {
            CustomMockServiceProvider {
                brew_service: Arc::new(CustomMockBrewService) as Arc<dyn BrewService>,
                link_service: Arc::new(CustomMockLinkService) as Arc<dyn LinkService>,
                load_service: Arc::new(CustomMockLoadService) as Arc<dyn LoadService>,
                deploy_service: Arc::new(CustomMockDeployService) as Arc<dyn DeployService>,
                fish_service: Arc::new(CustomMockFishService) as Arc<dyn FishService>,
                vscode_service: Arc::new(CustomMockVSCodeService) as Arc<dyn VSCodeService>,
            }
        }
    }

    impl ServiceProvider for CustomMockServiceProvider {
        fn brew_service(&self) -> Arc<dyn BrewService> {
            Arc::clone(&self.brew_service)
        }

        fn link_service(&self) -> Arc<dyn LinkService> {
            Arc::clone(&self.link_service)
        }

        fn load_service(&self) -> Arc<dyn LoadService> {
            Arc::clone(&self.load_service)
        }

        fn deploy_service(&self) -> Arc<dyn DeployService> {
            Arc::clone(&self.deploy_service)
        }

        fn fish_service(&self) -> Arc<dyn FishService> {
            Arc::clone(&self.fish_service)
        }

        fn vscode_service(&self) -> Arc<dyn VSCodeService> {
            Arc::clone(&self.vscode_service)
        }
    }

    #[tokio::test]
    async fn test_execute_fish_install() {
        let mock_services = Arc::new(CustomMockServiceProvider::new()) as Arc<dyn ServiceProvider>;

        let args = FishArgs {
            command: FishCommands::Install,
        };
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_fish_install_fisher() {
        let mock_services = Arc::new(CustomMockServiceProvider::new()) as Arc<dyn ServiceProvider>;

        let args = FishArgs {
            command: FishCommands::InstallFisher,
        };
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_fish_set_default() {
        let mock_services = Arc::new(CustomMockServiceProvider::new()) as Arc<dyn ServiceProvider>;

        let args = FishArgs {
            command: FishCommands::SetDefault,
        };
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());
    }
}
