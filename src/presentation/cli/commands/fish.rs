use crate::application::AppConfig;
use crate::error::AppError;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct FishArgs {
    #[clap(subcommand)]
    command: FishCommands,
}

#[derive(Subcommand)]
enum FishCommands {
    Install,
    SetDefault,
    InstallFisher,
}

pub async fn execute(args: FishArgs, config: &AppConfig) -> Result<(), AppError> {
    match args.command {
        FishCommands::Install => {
            config.get_fish_service().install().await?;
            println!("Fish shell installed successfully");
        }
        FishCommands::SetDefault => {
            config.get_fish_service().set_default().await?;
            println!("Fish shell set as default successfully");
        }
        FishCommands::InstallFisher => {
            config.get_fish_service().install_fisher().await?;
            println!("Fisher plugin manager installed successfully");
        }
    }
    Ok(())
}
