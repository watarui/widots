use crate::application::AppConfig;
use crate::error::AppError;
use clap::{ArgAction, Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    Link(commands::link::LinkArgs),
    Materialize(commands::materialize::MaterializeArgs),
    // todo implement read from yaml
    Brew(commands::brew::BrewArgs),
    Fish(commands::fish::FishArgs),
    VSCode(commands::vscode::VSCodeArgs),
}

pub async fn run(args: Args, config: &AppConfig) -> Result<(), AppError> {
    match args.command {
        Commands::Link(link_args) => commands::link::execute(link_args, config).await,
        Commands::Materialize(materialize_args) => {
            commands::materialize::execute(materialize_args, config).await
        }
        Commands::Brew(brew_args) => commands::brew::execute(brew_args, config).await,
        Commands::Fish(fish_args) => commands::fish::execute(fish_args, config).await,
        Commands::VSCode(vscode_args) => commands::vscode::execute(vscode_args, config).await,
    }
}
