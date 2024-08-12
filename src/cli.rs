use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Debug, Parser, Clone)]
#[command(name = "widots")]
#[command(author, version, about, long_about = None)]
#[command(color = clap::ColorChoice::Always)]
#[command(help_expected = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
    /// Output log
    #[arg(
      short,
      long,
      action = ArgAction::Count, help = "Increases verbosity level (use -vv for more verbosity)"
    )]
    pub verbose: u8,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    #[command(
        about = "Link dotfiles to home directory",
        long_about = "\
Link dotfiles to home directory.
Specify the path to the dotfiles directory
you want to link to the home directory."
    )]
    Link(Dotfiles),
    #[command(about = "Materialize dotfiles to destination directory")]
    Materialize(MaterializeDestination),
    #[command(
        about = "Execute procedures from yaml file",
        long_about = "\
Execute procedures from yaml file.
Specify the path to the yaml file.
By default, the file is ~/.config/widots/cofig.yml."
    )]
    Run(Yaml),
    #[command(about = "Execute brew operations")]
    Brew(BrewOperation),
    #[command(about = "Builds and deploys the executable to the local machine")]
    Deploy,
    #[command(about = "Excuting fish shell operations")]
    Fish(FishOperation),
    #[command(about = "Manage VSCode extensions")]
    Vscode(VSCodeExtensionOperation),
}

#[derive(Debug, Parser, Clone)]
pub struct Dotfiles {
    #[arg(
      value_hint = ValueHint::FilePath,
      help = "The path to the dotfiles directory",
      value_name = "DOTFILES_DIR_PATH",
    )]
    pub path: PathBuf,

    #[arg(
        short,
        long,
        help = "Force create symlinks, overwriting existing files"
    )]
    pub force: bool,

    #[arg(
        short,
        long,
        help = "Test the dotfiles directory for symlinks and files"
    )]
    pub test: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct MaterializeDestination {
    #[arg(
      value_hint = ValueHint::FilePath,
      help = "The path to the materialize destination directory",
      value_name = "MATERIALIZE_DESTINATION_PATH",
    )]
    pub path: PathBuf,
}

#[derive(Debug, Parser, Clone)]
pub struct Yaml {
    #[arg(
      value_hint = ValueHint::FilePath,
      help = "The path to the yaml file",
      default_value = "~/.config/widots/config.yml",
      value_name = "CONFIG_YAML_FILE_PATH",
    )]
    pub path: PathBuf,

    #[arg(
        short,
        long,
        help = "Force create symlinks, overwriting existing files"
    )]
    pub force: bool,

    #[arg(
        short,
        long,
        help = "Test the dotfiles directory for symlinks and files"
    )]
    pub test: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct BrewOperation {
    #[arg(value_enum, help = "Homebrew operation to execute")]
    pub action: BrewAction,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum BrewAction {
    Install,
    Import,
    Export,
}

#[derive(Debug, Parser, Clone)]
pub struct FishOperation {
    #[arg(value_enum, help = "Fish shell operation to execute")]
    pub action: FishAction,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum FishAction {
    Install,
    Default,
    Fisher,
}

#[derive(Debug, Parser, Clone)]
pub struct VSCodeExtensionOperation {
    #[arg(value_enum, help = "VSCode extension operation to execute")]
    pub action: VSCodeExtensionAction,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum VSCodeExtensionAction {
    Import,
    Export,
    Code,
}
