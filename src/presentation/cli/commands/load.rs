use crate::application::AppConfig;
use crate::error::AppError;
// use crate::models::link::FileProcessResult;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LoadArgs {
    #[clap(short, long)]
    target_yaml: PathBuf,
}

pub async fn execute(_args: LoadArgs, _config: &AppConfig) -> Result<(), AppError> {
    unimplemented!()
}
