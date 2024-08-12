use log::debug;

use crate::config::constants::{
    DEPLOY_DESTINATION_PATH, DEPLOY_SOURCE_PATH, FISH_COMPLETIONS_FILENAME,
    FISH_COMPLETIONS_SOURCE_PATH, FISH_COMPLETIONS_TARGET_DIR,
};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::{Argument, Cmd};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait Deployable {
    fn execute(&self) -> Result<(), AppError>;
    fn build_project(&self) -> Result<bool, AppError>;
    fn deploy_executable(&self) -> Result<(), AppError>;
    fn locate_fish_completions(&self) -> Result<(), AppError>;
}

pub struct Deployer {
    shell: Arc<dyn ShellOperations>,
}

impl Deployer {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }
}

impl Deployable for Deployer {
    fn execute(&self) -> Result<(), AppError> {
        println!("Building the project in release mode...");
        if !self.build_project()? {
            return Err(AppError::Deployment("Build failed".to_string()));
        }

        println!("Deploying the executable...");
        self.deploy_executable()?;
        println!("Deployment successful!");

        println!("Locating fish shell command completion files...");
        self.locate_fish_completions()?;
        println!("Locate successful!");

        Ok(())
    }

    fn build_project(&self) -> Result<bool, AppError> {
        let output = self.shell.shell(Cmd::Cmd("cargo build --release".into()))?;

        debug!("cargo build --release output: {:?}", output);

        Ok(output.status.success())
    }

    fn deploy_executable(&self) -> Result<(), AppError> {
        let source_path = Path::new(DEPLOY_SOURCE_PATH);
        let dest_path = Path::new(DEPLOY_DESTINATION_PATH);

        debug!(
            "Deploy from {:?} to {:?}",
            source_path.display(),
            dest_path.display()
        );

        if !source_path.exists() {
            return Err(AppError::FileNotFound(source_path.to_path_buf()));
        }

        self.shell.sudo(
            Cmd::Cmd("cp".into()),
            Argument::Args(vec![
                source_path.to_str().unwrap().into(),
                dest_path.to_str().unwrap().into(),
            ]),
        )?;

        self.shell.sudo(
            Cmd::Cmd("chmod".into()),
            Argument::Args(vec!["+x".into(), dest_path.to_str().unwrap().into()]),
        )?;

        Ok(())
    }

    fn locate_fish_completions(&self) -> Result<(), AppError> {
        let home = dirs::home_dir().ok_or(AppError::HomeDirectoryNotFound)?;
        // todo constants
        let source_path = PathBuf::from(FISH_COMPLETIONS_SOURCE_PATH);
        let dest_dir = home.join(FISH_COMPLETIONS_TARGET_DIR);
        let dest_path = dest_dir.join(FISH_COMPLETIONS_FILENAME);

        std::fs::create_dir_all(&dest_dir).map_err(|e| AppError::Io(Arc::new(e)))?;

        debug!(
            "Copy fish completions file from {:?} to {:?}",
            source_path.display(),
            dest_path.display()
        );

        std::fs::copy(&source_path, &dest_path).map_err(|e| AppError::Io(Arc::new(e)))?;
        Ok(())
    }
}
