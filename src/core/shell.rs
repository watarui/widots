use crate::error::app_error::AppError;
use crate::models::shell::{Argument, Cmd};
use async_trait::async_trait;
use log::debug;
use std::process::Output;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Provides operations for executing shell commands.
#[async_trait]
pub trait ShellOperations: Send + Sync {
    /// Executes a bash script.
    ///
    /// # Arguments
    ///
    /// * `script` - The bash script to execute
    ///
    /// # Returns
    ///
    /// The output of the executed script, or an error if execution fails.
    async fn bash(&self, script: &str) -> Result<Output, AppError>;

    /// Executes a shell command.
    ///
    /// # Arguments
    ///
    /// * `script` - The command to execute
    ///
    /// # Returns
    ///
    /// The output of the executed command, or an error if execution fails.
    async fn shell(&self, script: &Cmd<'_>) -> Result<Output, AppError>;

    /// Executes a command with sudo privileges.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `arg` - The arguments for the command
    ///
    /// # Returns
    ///
    /// The output of the executed command, or an error if execution fails.
    async fn sudo(&self, command: &Cmd<'_>, arg: &Argument<'_>) -> Result<Output, AppError>;

    /// Checks if a command exists in the system's PATH.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to check
    ///
    /// # Returns
    ///
    /// The output of the 'which' command, or an error if the command is not found.
    async fn which(&self, command: &Cmd<'_>) -> Result<Output, AppError>;
}

/// Executes shell commands.
pub struct ShellExecutor;

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellExecutor {
    /// Creates a new `ShellExecutor`.
    pub fn new() -> Self {
        ShellExecutor
    }
}

#[async_trait]
impl ShellOperations for ShellExecutor {
    async fn bash(&self, script: &str) -> Result<Output, AppError> {
        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| AppError::TempFileCreationError(e.to_string()))?;
        let temp_path = temp_file.path().to_owned();

        let mut file = File::create(&temp_path)
            .await
            .map_err(|e| AppError::ScriptWriteError(e.to_string()))?;
        file.write_all(script.as_bytes())
            .await
            .map_err(|e| AppError::ScriptWriteError(e.to_string()))?;

        let output = Command::new("bash")
            .arg(&temp_path)
            .output()
            .await
            .map_err(|e| AppError::BashExecutionFailed(e.to_string()))?;

        debug!("Executing bash command: {:?}", script);

        Ok(output)
    }

    async fn shell(&self, script: &Cmd<'_>) -> Result<Output, AppError> {
        match script {
            Cmd::Empty => Err(AppError::EmptyCommand),
            Cmd::Cmd(cmd) => {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(cmd.as_ref())
                    .output()
                    .await
                    .map_err(|e| AppError::ShellExecutionFailed(e.to_string()))?;

                debug!("Executing sh command: {:?}", cmd);

                Ok(output)
            }
        }
    }

    async fn sudo(&self, command: &Cmd<'_>, arg: &Argument<'_>) -> Result<Output, AppError> {
        let mut cmd = Command::new("sudo");

        match command {
            Cmd::Empty => return Err(AppError::EmptyCommand),
            Cmd::Cmd(c) => {
                cmd.arg(c.as_ref());
            }
        }

        match arg {
            Argument::Empty => {}
            Argument::Arg(a) => {
                cmd.arg(a.as_ref());
            }
            Argument::Args(args) => {
                cmd.args(args.iter().map(|a| a.as_ref()));
            }
        }

        debug!("Executing sudo command: {:?}", cmd);

        cmd.output()
            .await
            .map_err(|e| AppError::SudoExecutionFailed(e.to_string()))
    }

    async fn which(&self, command: &Cmd<'_>) -> Result<Output, AppError> {
        match command {
            Cmd::Empty => Err(AppError::EmptyCommand),
            Cmd::Cmd(cmd) => {
                let output = Command::new("which")
                    .arg(cmd.as_ref())
                    .output()
                    .await
                    .map_err(|e| AppError::WhichExecutionError(e.to_string()))?;

                debug!("Executing which command: {:?}", cmd);

                Ok(output)
            }
        }
    }
}
