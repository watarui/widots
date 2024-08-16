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
