use crate::application::service_provider::ServiceProvider;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use clap::{Args, ValueHint};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

static TEST_OUTPUT: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Args)]
pub struct MaterializeArgs {
    #[arg(
        value_hint = ValueHint::FilePath,
        help = "The path to the dotfiles directory to materialize",
        value_name = "TARGET_DOTFILES_DIR_PATH"
    )]
    target: PathBuf,
}

pub async fn execute(
    args: MaterializeArgs,
    services: &dyn ServiceProvider,
) -> Result<(), AppError> {
    let results = services
        .link_service()
        .materialize_dotfiles(&args.target)
        .await?;

    for result in results {
        if let FileProcessResult::Materialized(path, original) = result {
            let output = format!(
                "Materialized: {} (was linked to {})",
                path.display(),
                original.display()
            );
            if std::env::var("TEST_MODE").is_ok() {
                TEST_OUTPUT.lock().unwrap().push(output);
            } else {
                println!("{}", output);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_provider::ServiceProvider;
    use crate::application::services::brew_service::BrewService;
    use crate::application::services::deploy_service::DeployService;
    use crate::application::services::fish_service::FishService;
    use crate::application::services::link_service::LinkService;
    use crate::application::services::load_service::LoadService;
    use crate::application::services::vscode_service::VSCodeService;
    use crate::error::AppError;
    use crate::models::link::FileProcessResult;
    use crate::presentation::cli::commands::materialize::{execute, MaterializeArgs};
    use async_trait::async_trait;
    use std::path::{Path, PathBuf};
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

    struct CustomMockLinkService {
        materialize_result: Vec<FileProcessResult>,
    }

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
            Ok(self.materialize_result.clone())
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
        fn new(link_service: Arc<dyn LinkService>) -> Self {
            CustomMockServiceProvider {
                brew_service: Arc::new(CustomMockBrewService) as Arc<dyn BrewService>,
                link_service,
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
    async fn test_execute_materialize_dotfiles() {
        let mock_link_service = Arc::new(CustomMockLinkService {
            materialize_result: vec![],
        });
        let mock_services = Arc::new(CustomMockServiceProvider::new(
            Arc::clone(&mock_link_service) as Arc<dyn LinkService>,
        )) as Arc<dyn ServiceProvider>;

        let args = MaterializeArgs {
            target: PathBuf::new(),
        };
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_materialize_dotfiles_with_result() {
        let mock_link_service = Arc::new(CustomMockLinkService {
            materialize_result: vec![FileProcessResult::Materialized(
                PathBuf::from("/home/user/.bashrc"),
                PathBuf::from("/dotfiles/.bashrc"),
            )],
        });
        let mock_services = Arc::new(CustomMockServiceProvider::new(
            Arc::clone(&mock_link_service) as Arc<dyn LinkService>,
        )) as Arc<dyn ServiceProvider>;

        let args = MaterializeArgs {
            target: PathBuf::new(),
        };

        // Set environment variable to enable test mode
        std::env::set_var("TEST_MODE", "1");

        // Clear previous test output
        TEST_OUTPUT.lock().unwrap().clear();

        // Execute the function
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());

        // Get the captured output
        let output = TEST_OUTPUT.lock().unwrap().join("\n");

        // Unset environment variable
        std::env::remove_var("TEST_MODE");

        assert!(
            output.contains("Materialized: /home/user/.bashrc (was linked to /dotfiles/.bashrc)")
        );
    }
}
