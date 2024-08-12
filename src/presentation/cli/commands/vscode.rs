use crate::application::AppConfig;
use crate::error::AppError;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct VSCodeArgs {
    #[clap(subcommand)]
    command: VSCodeCommands,
}

#[derive(Subcommand)]
enum VSCodeCommands {
    ExportExtensions,
    ImportExtensions,
    EnsureCodeCommand,
}

pub async fn execute(args: VSCodeArgs, config: &AppConfig) -> Result<(), AppError> {
    match args.command {
        VSCodeCommands::ExportExtensions => {
            config.get_vscode_service().export_extensions().await?;
            println!("VSCode extensions exported successfully");
        }
        VSCodeCommands::ImportExtensions => {
            config.get_vscode_service().import_extensions().await?;
            println!("VSCode extensions imported successfully");
        }
        VSCodeCommands::EnsureCodeCommand => {
            config.get_vscode_service().ensure_code_command().await?;
            println!("VSCode 'code' command is now available");
        }
    }
    Ok(())
}
