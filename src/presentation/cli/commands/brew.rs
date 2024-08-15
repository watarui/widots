use crate::application::service_provider::ServiceProvider;
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

pub async fn execute(args: BrewArgs, services: &dyn ServiceProvider) -> Result<(), AppError> {
    match args.command {
        BrewCommands::Install => {
            services.brew_service().install().await?;
            println!("Homebrew installed successfully");
        }
        BrewCommands::Import => {
            services.brew_service().import().await?;
            println!("Homebrew packages imported successfully");
        }
        BrewCommands::Export => {
            services.brew_service().export().await?;
            println!("Homebrew packages exported successfully");
        }
    }
    Ok(())
}
