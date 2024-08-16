use crate::application::service_provider::ServiceProvider;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct MaterializeArgs {
    #[clap(
        short,
        long,
        help = "The path to the dotfiles directory to materialize",
        value_name = "TARGET_DOTFILES_DIR_PATH"
    )]
    target: PathBuf,
}

pub async fn execute(
    args: MaterializeArgs,
    services: &dyn ServiceProvider,
) -> Result<(), AppError> {
    let results = services
        .link_service()
        .materialize_dotfiles(&args.target)
        .await?;

    for result in results {
        if let FileProcessResult::Materialized(path, original) = result {
            println!(
                "Materialized: {} (was linked to {})",
                path.display(),
                original.display()
            );
        }
    }

    Ok(())
}
