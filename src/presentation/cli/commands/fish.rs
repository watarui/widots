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
