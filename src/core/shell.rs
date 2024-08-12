use crate::error::app_error::AppError;
use crate::models::shell::{Argument, Cmd};
use log::debug;
use std::io::Write;
use std::ops::Deref;
use std::process::{Command, Output};
use std::sync::Arc;
use tempfile::NamedTempFile;

pub trait ShellOperations: Send + Sync {
    fn bash(&self, script: &str) -> Result<Output, AppError>;
    fn shell(&self, script: Cmd) -> Result<Output, AppError>;
    fn sudo(&self, command: Cmd, arg: Argument) -> Result<Output, AppError>;
    fn which(&self, command: Cmd) -> Result<Output, AppError>;
}

pub struct ShellExecutor;

impl Default for ShellExecutor {
    fn default() -> Self {
        ShellExecutor::new()
    }
}

impl ShellExecutor {
    pub fn new() -> Self {
        ShellExecutor
    }
}

impl ShellOperations for ShellExecutor {
    fn bash(&self, script: &str) -> Result<Output, AppError> {
        let mut temp_file =
            NamedTempFile::new().map_err(|e| AppError::TempFileCreationError(e.to_string()))?;
        temp_file
            .write_all(script.as_bytes())
            .map_err(|e| AppError::ScriptWriteError(e.to_string()))?;

        let mut bash = Command::new("bash");
        let cmd = bash.arg(temp_file.path());

        debug!("Executing bash command: {:?}", cmd);

        cmd.output()
            .map_err(|e| AppError::BashExecutionFailed(e.to_string()))
    }

    fn shell(&self, script: Cmd) -> Result<Output, AppError> {
        match script {
            Cmd::Empty => Err(AppError::EmptyCommand),
            Cmd::Cmd(cmd) => {
                let mut sh = Command::new("sh");
                let c = sh.arg("-c").arg(cmd.as_ref());

                debug!("Executing sh command: {:?}", c);

                c.output()
                    .map_err(|e| AppError::ShellExecutionFailed(e.to_string()))
            }
        }
    }

    fn sudo(&self, command: Cmd, arg: Argument) -> Result<Output, AppError> {
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
            .map_err(|e| AppError::SudoExecutionFailed(e.to_string()))
    }

    fn which(&self, command: Cmd) -> Result<Output, AppError> {
        match command {
            Cmd::Empty => Err(AppError::EmptyCommand),
            Cmd::Cmd(cmd) => {
                let mut which = Command::new("which");
                let c = which.arg(cmd.as_ref());

                debug!("Executing which command: {:?}", c);

                c.output()
                    .map_err(|e| AppError::WhichExecutionError(e.to_string()))
            }
        }
    }
}

impl<T: ShellOperations + ?Sized> ShellOperations for Arc<T> {
    fn bash(&self, script: &str) -> Result<Output, AppError> {
        self.deref().bash(script)
    }

    fn shell(&self, script: Cmd) -> Result<Output, AppError> {
        self.deref().shell(script)
    }

    fn sudo(&self, command: Cmd, arg: Argument) -> Result<Output, AppError> {
        self.deref().sudo(command, arg)
    }

    fn which(&self, command: Cmd) -> Result<Output, AppError> {
        self.deref().which(command)
    }
}
