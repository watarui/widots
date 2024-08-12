use crate::application::AppConfig;
use crate::error::AppError;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LinkArgs {
    #[clap(short, long)]
    source: PathBuf,

    #[clap(short, long)]
    target: PathBuf,

    #[clap(short, long)]
    force: bool,
}

pub async fn execute(args: LinkArgs, config: &AppConfig) -> Result<(), AppError> {
    let results = config
        .get_link_service()
        .link_dotfiles(&args.source, &args.target, args.force)
        .await?;

    for result in results {
        match result {
            crate::models::link::FileProcessResult::Linked(src, dst) => {
                println!("Linked: {} -> {}", src.display(), dst.display());
            }
            crate::models::link::FileProcessResult::Created(path) => {
                println!("Created directory: {}", path.display());
            }
            crate::models::link::FileProcessResult::Skipped(path) => {
                println!("Skipped: {}", path.display());
            }
            crate::models::link::FileProcessResult::_Error(e) => {
                println!("Error: {:?}", e);
            }
            _ => {}
        }
    }

    Ok(())
}
