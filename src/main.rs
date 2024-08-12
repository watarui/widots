use clap::Parser;
use log::LevelFilter;
use widots::{
    cli::{
        Args, BrewAction, BrewOperation, Commands, Dotfiles, FishAction, FishOperation,
        MaterializeDestination, VSCodeExtensionAction, VSCodeExtensionOperation, Yaml,
    },
    commands::{
        brew::HomebrewOperations, deploy::Deployable, fish::FishOperations,
        fisher::FisherOperations, materialize::Materializable, run::RunnerOperations,
        vscode::VSCodeOperations,
    },
    config::app_config::AppConfig,
    error::app_error::AppError,
    logger::log::setup_logger,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug, // -v
        _ => LevelFilter::Trace, // -vv
    };

    setup_logger(log_level).map_err(|e| AppError::Logger(e.to_string()))?;

    let config = AppConfig::new().await?;

    match args.command {
        Commands::Link(dots) => link_command(&config, dots).await?,
        Commands::Materialize(dest) => materialize_command(&config, dest).await?,
        Commands::Run(yaml) => run_command(&config, yaml).await?,
        Commands::Brew(op) => brew_command(&config, op).await?,
        Commands::Deploy => deploy_command(&config).await?,
        Commands::Fish(op) => fish_command(&config, op).await?,
        Commands::Vscode(op) => vscode_command(&config, op).await?,
    }

    Ok(())
}

async fn link_command(config: &AppConfig, dots: Dotfiles) -> Result<(), AppError> {
    let linker = config.linker();
    linker
        .link_recursively(&dots.path, &dirs::home_dir().unwrap(), dots.force)
        .await?;
    Ok(())
}

async fn materialize_command(
    config: &AppConfig,
    dest: MaterializeDestination,
) -> Result<(), AppError> {
    let materializer = config.materializer();
    materializer.execute(&dest.path).await?;
    Ok(())
}

async fn run_command(config: &AppConfig, yaml: Yaml) -> Result<(), AppError> {
    let runner = config.runner();
    runner.execute(&yaml.path, yaml.force).await?;
    Ok(())
}

async fn brew_command(config: &AppConfig, op: BrewOperation) -> Result<(), AppError> {
    let homebrew = config.homebrew();
    match op.action {
        BrewAction::Install => homebrew.install().await?,
        BrewAction::Import => homebrew.import().await?,
        BrewAction::Export => homebrew.export().await?,
    }
    Ok(())
}

async fn deploy_command(config: &AppConfig) -> Result<(), AppError> {
    let deployer = config.deployer();
    deployer.execute().await?;
    Ok(())
}

async fn fish_command(config: &AppConfig, op: FishOperation) -> Result<(), AppError> {
    let fish = config.fish();
    match op.action {
        FishAction::Install => fish.install().await?,
        FishAction::Default => fish.set_default().await?,
        FishAction::Fisher => {
            let fisher = config.fisher();
            fisher.install().await?;
        }
    }
    Ok(())
}

async fn vscode_command(config: &AppConfig, op: VSCodeExtensionOperation) -> Result<(), AppError> {
    let vscode = config.vscode();
    match op.action {
        VSCodeExtensionAction::Import => vscode.import().await?,
        VSCodeExtensionAction::Export => vscode.export().await?,
        VSCodeExtensionAction::Code => vscode.ensure_code_command().await?,
    }
    Ok(())
}
