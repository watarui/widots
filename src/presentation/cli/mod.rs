use crate::application::service_provider::ProductionServiceProvider;
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

    #[clap(short, long, global = true, action = ArgAction::Count)]
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
    VSCode(commands::vscode::VSCodeArgs),
}

pub async fn run(args: Args, service_provider: &ProductionServiceProvider) -> Result<(), AppError> {
    match args.command {
        Commands::Link(link_args) => commands::link::execute(link_args, service_provider).await,
        Commands::Materialize(materialize_args) => {
            commands::materialize::execute(materialize_args, service_provider).await
        }
        Commands::Load(load_args) => commands::load::execute(load_args, service_provider).await,
        Commands::Deploy => commands::deploy::execute(service_provider).await,
        Commands::Brew(brew_args) => commands::brew::execute(brew_args, service_provider).await,
        Commands::Fish(fish_args) => commands::fish::execute(fish_args, service_provider).await,
        Commands::VSCode(vscode_args) => {
            commands::vscode::execute(vscode_args, service_provider).await
        }
    }
}
