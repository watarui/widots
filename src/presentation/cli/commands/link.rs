use crate::application::service_provider::ServiceProvider;
use crate::constants::TEST_HOME_DIR;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use clap::{Args, ValueHint};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

static TEST_OUTPUT: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Args)]
pub struct LinkArgs {
    #[arg(
        value_hint = ValueHint::FilePath,
        help = "The path to the dotfiles directory",
        value_name = "SOURCE_DOTFILES_DIR_PATH"
    )]
    source_path: PathBuf,

    #[arg(
        short,
        long,
        help = "Link to the test directory instead of the home directory for testing purposes"
    )]
    test: bool,
}

pub async fn execute(args: LinkArgs, services: &dyn ServiceProvider) -> Result<(), AppError> {
    let home = dirs::home_dir().ok_or(AppError::DirectoryNotFound)?;
    let target = if args.test {
        home.join(TEST_HOME_DIR)
    } else {
        home
    };

    let results = services
        .link_service()
        .link_dotfiles(&args.source_path, &target)
        .await?;

    for result in results {
        match result {
            FileProcessResult::Linked(src, dst) => {
                let output = format!("Linked: {} -> {}", src.display(), dst.display());
                if std::env::var("TEST_MODE").is_ok() {
                    TEST_OUTPUT.lock().unwrap().push(output);
                } else {
                    println!("{}", output);
                }
            }
            FileProcessResult::Created(path) => {
                let output = format!("Created directory: {}", path.display());
                if std::env::var("TEST_MODE").is_ok() {
                    TEST_OUTPUT.lock().unwrap().push(output);
                } else {
                    println!("{}", output);
                }
            }
            FileProcessResult::Skipped(path) => {
                let output = format!("Skipped: {}", path.display());
                if std::env::var("TEST_MODE").is_ok() {
                    TEST_OUTPUT.lock().unwrap().push(output);
                } else {
                    println!("{}", output);
                }
            }
            FileProcessResult::Materialized(_, _) => {} // This should not occur during linking
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
    use crate::presentation::cli::commands::link::{execute, LinkArgs};
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
        results: Vec<FileProcessResult>,
    }

    #[async_trait]
    impl LinkService for CustomMockLinkService {
        async fn link_dotfiles(
            &self,
            _source: &Path,
            _target: &Path,
        ) -> Result<Vec<FileProcessResult>, AppError> {
            Ok(self.results.clone())
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
    async fn test_execute_link_dotfiles() {
        let mock_services = Arc::new(CustomMockServiceProvider::new(Arc::new(
            CustomMockLinkService {
                results: vec![FileProcessResult::Linked(
                    PathBuf::from("/src/file"),
                    PathBuf::from("/dst/file"),
                )],
            },
        ))) as Arc<dyn ServiceProvider>;

        let args = LinkArgs {
            source_path: PathBuf::new(),
            test: false,
        };
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_link_dotfiles_with_test() {
        let mock_services = Arc::new(CustomMockServiceProvider::new(Arc::new(
            CustomMockLinkService {
                results: vec![FileProcessResult::Linked(
                    PathBuf::from("/src/file"),
                    PathBuf::from("/dst/file"),
                )],
            },
        ))) as Arc<dyn ServiceProvider>;

        let args = LinkArgs {
            source_path: PathBuf::new(),
            test: true,
        };
        let result = execute(args, mock_services.as_ref()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_link_dotfiles_with_various_results() {
        let mock_link_service = Arc::new(CustomMockLinkService {
            results: vec![
                FileProcessResult::Linked(PathBuf::from("/src/file1"), PathBuf::from("/dst/file1")),
                FileProcessResult::Created(PathBuf::from("/dst/dir1")),
                FileProcessResult::Skipped(PathBuf::from("/src/file2")),
                FileProcessResult::Materialized(
                    PathBuf::from("/src/file3"),
                    PathBuf::from("/dst/file3"),
                ),
            ],
        });
        let mock_services = Arc::new(CustomMockServiceProvider::new(
            Arc::clone(&mock_link_service) as Arc<dyn LinkService>,
        )) as Arc<dyn ServiceProvider>;

        let args = LinkArgs {
            source_path: PathBuf::new(),
            test: true,
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

        // Assert the output contains the expected messages
        assert!(output.contains("Linked: /src/file1 -> /dst/file1"));
        assert!(output.contains("Created directory: /dst/dir1"));
        assert!(output.contains("Skipped: /src/file2"));
        // Materialized case should not produce any output
        assert!(!output.contains("Materialized"));
    }
}
