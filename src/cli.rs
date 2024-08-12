use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum, ValueHint};

/// CLI arguments
#[derive(Debug, Parser, Clone)]
#[command(name = "widots")]
#[command(author, version, about, long_about = None)]
#[command(color = clap::ColorChoice::Always)]
#[command(help_expected = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
    /// Verbosity level
    #[arg(
      short,
      long,
      action = ArgAction::Count, help = "Increases verbosity level (use -vv for more verbosity)"
    )]
    pub verbose: u8,
}

/// subcommands
#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    /// Link dotfiles
    #[command(
        about = "Link dotfiles to home directory",
        long_about = "\
Link dotfiles to home directory.
Specify the path to the dotfiles directory
you want to link to the home directory."
    )]
    Link(Dotfiles),
    /// Materialize dotfiles
    #[command(about = "Materialize dotfiles to destination directory")]
    Materialize(MaterializeDestination),
    /// Run configuration
    #[command(
        about = "Execute procedures from yaml file",
        long_about = "\
Execute procedures from yaml file.
Specify the path to the yaml file.
By default, the file is ~/.config/widots/cofig.yml."
    )]
    Run(Yaml),
    /// Manage Homebrew
    #[command(about = "Execute brew operations")]
    Brew(BrewOperation),
    /// Deploy application
    #[command(about = "Builds and deploys the executable to the local machine")]
    Deploy,
    /// Manage Fish shell
    #[command(about = "Excuting fish shell operations")]
    Fish(FishOperation),
    /// Manage VSCode extensions
    #[command(about = "Manage VSCode extensions")]
    Vscode(VSCodeExtensionOperation),
}

/// Dotfiles linking options
#[derive(Debug, Parser, Clone)]
pub struct Dotfiles {
    /// Path to dotfiles
    #[arg(
      value_hint = ValueHint::FilePath,
      help = "The path to the dotfiles directory",
      value_name = "DOTFILES_DIR_PATH",
    )]
    pub path: PathBuf,
    /// Force overwrite existing files
    #[arg(
        short,
        long,
        help = "Force create symlinks, overwriting existing files"
    )]
    pub force: bool,
    /// Test the dotfiles directory for symlinks and files
    #[arg(
        short,
        long,
        help = "Test the dotfiles directory for symlinks and files"
    )]
    pub test: bool,
}

/// Materialize destination options
#[derive(Debug, Parser, Clone)]
pub struct MaterializeDestination {
    /// Path to materialize destination
    #[arg(
      value_hint = ValueHint::FilePath,
      help = "The path to the materialize destination directory",
      value_name = "MATERIALIZE_DESTINATION_PATH",
    )]
    pub path: PathBuf,
}

/// YAML configuration options
#[derive(Debug, Parser, Clone)]
pub struct Yaml {
    /// Path to YAML configuration file
    #[arg(
      value_hint = ValueHint::FilePath,
      help = "The path to the yaml file",
      default_value = "~/.config/widots/config.yml",
      value_name = "CONFIG_YAML_FILE_PATH",
    )]
    pub path: PathBuf,
    /// Force overwrite existing files
    #[arg(
        short,
        long,
        help = "Force create symlinks, overwriting existing files"
    )]
    pub force: bool,
    /// Test the dotfiles directory for symlinks and files
    #[arg(
        short,
        long,
        help = "Test the dotfiles directory for symlinks and files"
    )]
    pub test: bool,
}

/// Homebrew operation options
#[derive(Debug, Parser, Clone)]
pub struct BrewOperation {
    /// Homebrew action to perform
    #[arg(value_enum, help = "Homebrew operation to execute")]
    pub action: BrewAction,
}

/// Homebrew actions
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum BrewAction {
    Install,
    Import,
    Export,
}

/// Fish shell operation options
#[derive(Debug, Parser, Clone)]
pub struct FishOperation {
    /// Fish shell action to perform
    #[arg(value_enum, help = "Fish shell operation to execute")]
    pub action: FishAction,
}

/// Fish shell actions
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum FishAction {
    Install,
    Default,
    Fisher,
}

/// VSCode extension operation options
#[derive(Debug, Parser, Clone)]
pub struct VSCodeExtensionOperation {
    /// VSCode extension action to perform
    #[arg(value_enum, help = "VSCode extension operation to execute")]
    pub action: VSCodeExtensionAction,
}

/// VSCode extension actions
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum VSCodeExtensionAction {
    Import,
    Export,
    Code,
}
