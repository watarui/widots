use crate::application::AppConfig;
use crate::constants::DEFAULT_CONFIG_TOML;
use crate::constants::TEST_HOME_DIR;
use crate::error::AppError;
use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args)]
pub struct LoadArgs {
    #[arg(
    value_hint = ValueHint::FilePath,
    help = "The path to the TOML file",
    default_value = DEFAULT_CONFIG_TOML,
    value_name = "CONFIG_TOML_FILE_PATH"
    )]
    config_toml: PathBuf,

    #[arg(
        short,
        long,
        help = "Force create symlinks, overwriting existing files"
    )]
    force: bool,

    #[arg(
        short,
        long,
        help = "Link to the test directory instead of the home directory for testing purposes"
    )]
    test: bool,
}

pub async fn execute(args: LoadArgs, config: &AppConfig) -> Result<(), AppError> {
    let home = dirs::home_dir().ok_or(AppError::DirectoryNotFound)?;
    let target = if args.test {
        home.join(TEST_HOME_DIR)
    } else {
        home
    };

    config
        .get_load_service()
        .load(&args.config_toml, &target, args.force)
        .await
}
