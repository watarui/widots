use clap::Parser;
use log::LevelFilter::{Debug, Info, Trace};
use std::process::exit;
use widots::{
    cli::{Args, BrewAction, Commands, FishAction, VSCodeExtensionAction},
    commands::{
        brew::{Homebrew, HomebrewOperations},
        deploy::{Deployable, Deployer},
        fish::{Fish, FishOperations},
        fisher::{Fisher, FisherOperations},
        link::{make_symlinks, Linker},
        materialize::{Materializable, Materializer},
        run::{Runner, RunnerOperations},
        vscode::{VSCode, VSCodeOperations},
    },
    config::app_config::AppConfig,
    create_app_config,
    error::app_error::AppError,
    logger::log::setup_logger,
    Result,
};

fn run(args: Args, config: &AppConfig) -> Result<()> {
    match args.command {
        Commands::Link(dots) => {
            let linker = Linker::new();
            make_symlinks(linker, &dots.path, dots.force, dots.test)?;
        }
        Commands::Materialize(dest) => {
            let materializer = Materializer::new(Linker::new());
            materializer.execute(&dest.path)?;
        }
        Commands::Run(yaml) => {
            let runner = Runner::new(
                config.shell_executor.clone(),
                config.yaml_parser.clone(),
                Linker::new(),
            );
            runner.execute(&yaml.path, yaml.force)?;
        }
        Commands::Brew(op) => {
            let homebrew = Homebrew::new(config.shell_executor.clone());
            match op.action {
                BrewAction::Install => homebrew.install()?,
                BrewAction::Import => homebrew.import()?,
                BrewAction::Export => homebrew.export()?,
            }
        }
        Commands::Deploy => {
            let deployer = Deployer::new(config.shell_executor.clone());
            deployer.execute()?;
        }
        Commands::Fish(op) => {
            let fish = Fish::new(config.shell_executor.clone());
            match op.action {
                FishAction::Install => fish.install()?,
                FishAction::Default => fish.set_default()?,
                FishAction::Fisher => {
                    let fisher = Fisher::new(config.shell_executor.clone());
                    fisher.install()?;
                }
            }
        }
        Commands::Vscode(op) => {
            let vscode = VSCode::new(config.shell_executor.clone());
            match op.action {
                VSCodeExtensionAction::Import => vscode.import()?,
                VSCodeExtensionAction::Export => vscode.export()?,
                VSCodeExtensionAction::Code => vscode.ensure_code_command()?,
            }
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => Info,  // default
        1 => Debug, // -v
        _ => Trace, // -vv
    };

    let _ = setup_logger(log_level).map_err(|e| AppError::Logger(e.to_string()));

    let config = create_app_config();

    let result = run(args, &config);
    match result {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
