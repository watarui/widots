use async_trait::async_trait;
use log::debug;
use tokio::fs::{copy, create_dir_all};

use crate::config::constants::{
    DEPLOY_DESTINATION_PATH, DEPLOY_SOURCE_PATH, FISH_COMPLETIONS_FILENAME,
    FISH_COMPLETIONS_SOURCE_PATH, FISH_COMPLETIONS_TARGET_DIR,
};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::{Argument, Cmd};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[async_trait]
pub trait Deployable: Send + Sync {
    async fn execute(&self) -> Result<(), AppError>;
    async fn build_project(&self) -> Result<bool, AppError>;
    async fn deploy_executable(&self) -> Result<(), AppError>;
    async fn locate_fish_completions(&self) -> Result<(), AppError>;
}

pub struct Deployer {
    shell: Arc<dyn ShellOperations>,
}

impl Deployer {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }
}

#[async_trait]
impl Deployable for Deployer {
    async fn execute(&self) -> Result<(), AppError> {
        println!("Building the project in release mode...");
        if !self.build_project().await? {
            return Err(AppError::Deployment("Build failed".to_string()));
        }

        println!("Deploying the executable...");
        self.deploy_executable().await?;
        println!("Deployment successful!");

        println!("Locating fish shell command completion files...");
        self.locate_fish_completions().await?;
        println!("Locate successful!");

        Ok(())
    }

    async fn build_project(&self) -> Result<bool, AppError> {
        let cmd = Cmd::new("cargo build --release");
        let output = self.shell.shell(&cmd).await?;

        debug!("cargo build --release output: {:?}", output);

        Ok(output.status.success())
    }

    async fn deploy_executable(&self) -> Result<(), AppError> {
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

        let cmd = Cmd::new("cp");
        let args = Argument::Args(vec![
            source_path.to_str().unwrap().into(),
            dest_path.to_str().unwrap().into(),
        ]);
        self.shell.sudo(&cmd, &args).await?;

        let cmd = Cmd::new("chmod");
        let args = Argument::Args(vec!["+x".into(), dest_path.to_str().unwrap().into()]);
        self.shell.sudo(&cmd, &args).await?;

        Ok(())
    }

    async fn locate_fish_completions(&self) -> Result<(), AppError> {
        let home = dirs::home_dir().ok_or(AppError::HomeDirectoryNotFound)?;
        let source_path = PathBuf::from(FISH_COMPLETIONS_SOURCE_PATH);
        let dest_dir = home.join(FISH_COMPLETIONS_TARGET_DIR);
        let dest_path = dest_dir.join(FISH_COMPLETIONS_FILENAME);

        create_dir_all(&dest_dir).await?;

        debug!(
            "Copy fish completions file from {:?} to {:?}",
            source_path.display(),
            dest_path.display()
        );

        copy(&source_path, &dest_path).await?;
        Ok(())
    }
}
