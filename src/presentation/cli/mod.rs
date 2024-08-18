use crate::application::service_provider::ServiceProvider;
use crate::error::AppError;
use clap::{ArgAction, Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "widots")]
#[command(author, version, about, long_about = None)]
#[command(color = clap::ColorChoice::Always)]
#[command(help_expected = true)]
pub struct Args {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long, global = true, action = ArgAction::Count, help = "Sets the level of verbosity")]
    pub verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Link dotfiles to home directory")]
    Link(commands::link::LinkArgs),
    #[command(about = "Materialize dotfiles to destination directory")]
    Materialize(commands::materialize::MaterializeArgs),
    #[command(about = "Execute procedures from TOML file")]
    Load(commands::load::LoadArgs),
    #[command(about = "Builds and deploys the executable to the local machine")]
    Deploy,
    #[command(about = "Manage Homebrew packages")]
    Brew(commands::brew::BrewArgs),
    #[command(about = "Executing fish shell operations")]
    Fish(commands::fish::FishArgs),
    #[command(about = "Manage VSCode extensions")]
    Vscode(commands::vscode::VSCodeArgs),
}

pub async fn run<S: ServiceProvider>(args: Args, service_provider: &S) -> Result<(), AppError> {
    match args.command {
        Commands::Link(link_args) => commands::link::execute(link_args, service_provider).await,
        Commands::Materialize(materialize_args) => {
            commands::materialize::execute(materialize_args, service_provider).await
        }
        Commands::Load(load_args) => commands::load::execute(load_args, service_provider).await,
        Commands::Deploy => commands::deploy::execute(service_provider).await,
        Commands::Brew(brew_args) => commands::brew::execute(brew_args, service_provider).await,
        Commands::Fish(fish_args) => commands::fish::execute(fish_args, service_provider).await,
        Commands::Vscode(vscode_args) => {
            commands::vscode::execute(vscode_args, service_provider).await
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::application::services::brew_service::BrewService;
    use crate::application::services::deploy_service::DeployService;
    use crate::application::services::fish_service::FishService;
    use crate::application::services::link_service::LinkService;
    use crate::application::services::load_service::LoadService;
    use crate::application::services::vscode_service::VSCodeService;
    use crate::models::link::FileProcessResult;
    use async_trait::async_trait;
    use clap::Parser;
    use mockall::predicate::*;
    use mockall::*;
    use std::path::Path;

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

    mock! {
        pub ServiceProvider {}

        #[async_trait]
        impl ServiceProvider for ServiceProvider {
            fn link_service(&self) -> Arc<dyn LinkService>;
            fn load_service(&self) -> Arc<dyn LoadService>;
            fn deploy_service(&self) -> Arc<dyn DeployService>;
            fn brew_service(&self) -> Arc<dyn BrewService>;
            fn fish_service(&self) -> Arc<dyn FishService>;
            fn vscode_service(&self) -> Arc<dyn VSCodeService>;
        }
    }

    const APP: &str = "widots";

    #[tokio::test]
    async fn test_run_link_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_link_service()
            .returning(|| Arc::new(CustomMockLinkService));

        let args = Args::parse_from([APP, "link", "--test", "/src"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_materialize_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_link_service()
            .returning(|| Arc::new(CustomMockLinkService));

        let args = Args::parse_from([APP, "materialize", "/dst"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_load_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_load_service()
            .returning(|| Arc::new(CustomMockLoadService));

        let args = Args::parse_from([APP, "load", "--test", "/path/to/config"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_deploy_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_deploy_service()
            .returning(|| Arc::new(CustomMockDeployService));

        let args = Args::parse_from([APP, "deploy"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_brew_install_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_brew_service()
            .returning(|| Arc::new(CustomMockBrewService));

        let args = Args::parse_from([APP, "brew", "install"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_brew_import_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_brew_service()
            .returning(|| Arc::new(CustomMockBrewService));

        let args = Args::parse_from([APP, "brew", "import"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_brew_export_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_brew_service()
            .returning(|| Arc::new(CustomMockBrewService));

        let args = Args::parse_from([APP, "brew", "export"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fish_install_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_fish_service()
            .returning(|| Arc::new(CustomMockFishService));

        let args = Args::parse_from([APP, "fish", "install"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fish_default_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_fish_service()
            .returning(|| Arc::new(CustomMockFishService));

        let args = Args::parse_from([APP, "fish", "default"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fish_fisher_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_fish_service()
            .returning(|| Arc::new(CustomMockFishService));

        let args = Args::parse_from([APP, "fish", "fisher"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_vscode_export_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_vscode_service()
            .returning(|| Arc::new(CustomMockVSCodeService));

        let args = Args::parse_from([APP, "vscode", "export"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_vscode_import_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_vscode_service()
            .returning(|| Arc::new(CustomMockVSCodeService));

        let args = Args::parse_from([APP, "vscode", "import"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_vscode_code_command() {
        let mut mock_service_provider = MockServiceProvider::new();

        mock_service_provider
            .expect_vscode_service()
            .returning(|| Arc::new(CustomMockVSCodeService));

        let args = Args::parse_from([APP, "vscode", "code"]);

        let result = run(args, &mock_service_provider).await;
        assert!(result.is_ok());
    }
}
