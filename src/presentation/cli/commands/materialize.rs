use crate::application::AppConfig;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct MaterializeArgs {
    #[clap(short, long)]
    target: PathBuf,
}

pub async fn execute(args: MaterializeArgs, config: &AppConfig) -> Result<(), AppError> {
    let results = config
        .get_link_service()
        .materialize_dotfiles(&args.target)
        .await?;

    for result in results {
        match result {
            FileProcessResult::Materialized(path, original) => {
                println!(
                    "Materialized: {} (was linked to {})",
                    path.display(),
                    original.display()
                );
            }
            FileProcessResult::_Error(e) => {
                println!("Error: {:?}", e);
            }
            _ => {}
        }
    }

    Ok(())
}
