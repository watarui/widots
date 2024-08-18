use crate::application::service_provider::ServiceProvider;
use crate::constants::TEST_HOME_DIR;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use clap::{Args, ValueHint};
use std::path::PathBuf;

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
                println!("Linked: {} -> {}", src.display(), dst.display());
            }
            FileProcessResult::Created(path) => {
                println!("Created directory: {}", path.display());
            }
            FileProcessResult::Skipped(path) => {
                println!("Skipped: {}", path.display());
            }
            FileProcessResult::Materialized(_, _) => {} // This should not occur during linking
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::brew_service::BrewService;
    use crate::application::services::deploy_service::DeployService;
    use crate::application::services::fish_service::FishService;
    use crate::application::services::link_service::LinkService;
    use crate::application::services::load_service::LoadService;
    use crate::application::services::vscode_service::VSCodeService;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tempfile::TempDir;

    mock! {
        pub ServiceProvider {}
        impl ServiceProvider for ServiceProvider {
            fn brew_service(&self) -> Arc<dyn BrewService>;
            fn link_service(&self) -> Arc<dyn LinkService>;
            fn load_service(&self) -> Arc<dyn LoadService>;
            fn deploy_service(&self) -> Arc<dyn DeployService>;
            fn fish_service(&self) -> Arc<dyn FishService>;
            fn vscode_service(&self) -> Arc<dyn VSCodeService>;
        }
    }

    mock! {
        pub LinkService {}
        #[async_trait]
        impl LinkService for LinkService {
            async fn link_dotfiles(&self, source: &Path, target: &Path) -> Result<Vec<FileProcessResult>, AppError>;
            async fn materialize_dotfiles(&self, target: &Path) -> Result<Vec<FileProcessResult>, AppError>;
        }
    }

    #[tokio::test]
    async fn test_execute_link_dotfiles() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source");
        std::fs::create_dir(&source_path).unwrap();

        let mut mock_link_service = MockLinkService::new();
        mock_link_service
            .expect_link_dotfiles()
            .with(eq(source_path.clone()), always())
            .returning(|_, _| {
                Ok(vec![
                    FileProcessResult::Linked(
                        PathBuf::from("/src/file1"),
                        PathBuf::from("/dst/file1"),
                    ),
                    FileProcessResult::Created(PathBuf::from("/dst/dir1")),
                    FileProcessResult::Skipped(PathBuf::from("/src/file2")),
                    FileProcessResult::Materialized(
                        PathBuf::from("/src/file3"),
                        PathBuf::from("/dst/file3"),
                    ),
                ])
            });

        let mut mock_service_provider = MockServiceProvider::new();
        mock_service_provider
            .expect_link_service()
            .return_const(Arc::new(mock_link_service) as Arc<dyn LinkService>);

        let args = LinkArgs {
            source_path: source_path.clone(),
            test: false,
        };

        let result = execute(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_link_dotfiles_with_test_flag() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source");
        std::fs::create_dir(&source_path).unwrap();

        let mut mock_link_service = MockLinkService::new();
        mock_link_service
            .expect_link_dotfiles()
            .with(eq(source_path.clone()), always())
            .returning(|_, _| {
                Ok(vec![
                    FileProcessResult::Linked(
                        PathBuf::from("/src/file1"),
                        PathBuf::from("/dst/file1"),
                    ),
                    FileProcessResult::Created(PathBuf::from("/dst/dir1")),
                    FileProcessResult::Skipped(PathBuf::from("/src/file2")),
                ])
            });

        let mut mock_service_provider = MockServiceProvider::new();
        mock_service_provider
            .expect_link_service()
            .return_const(Arc::new(mock_link_service) as Arc<dyn LinkService>);

        let args = LinkArgs {
            source_path: source_path.clone(),
            test: true,
        };

        let result = execute(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }
}
