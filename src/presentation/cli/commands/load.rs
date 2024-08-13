use crate::application::AppConfig;
use crate::config::constants::TEST_HOME_DIR;
use crate::error::AppError;
// use crate::models::link::FileProcessResult;
use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args)]
pub struct LoadArgs {
    #[arg(
    value_hint = ValueHint::FilePath,
    help = "The path to the yaml file",
    default_value = "~/.config/widots/config.yml",
    value_name = "CONFIG_YAML_FILE_PATH"
    )]
    target_yaml: PathBuf,

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
    let home = dirs::home_dir().unwrap();
    let target = if args.test {
        home.join(TEST_HOME_DIR)
    } else {
        home
    };

    config
        .get_load_service()
        .load_yaml(&args.target_yaml, &target, args.force)
        .await
}
