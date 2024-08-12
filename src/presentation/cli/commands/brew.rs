use crate::application::AppConfig;
use crate::error::AppError;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct BrewArgs {
    #[clap(subcommand)]
    command: BrewCommands,
}

#[derive(Subcommand)]
enum BrewCommands {
    Install,
    Import,
    Export,
}

pub async fn execute(args: BrewArgs, config: &AppConfig) -> Result<(), AppError> {
    match args.command {
        BrewCommands::Install => {
            config.get_brew_service().install().await?;
            println!("Homebrew installed successfully");
        }
        BrewCommands::Import => {
            config.get_brew_service().import().await?;
            println!("Homebrew packages imported successfully");
        }
        BrewCommands::Export => {
            config.get_brew_service().export().await?;
            println!("Homebrew packages exported successfully");
        }
    }
    Ok(())
}
