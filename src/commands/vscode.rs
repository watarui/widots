use log::debug;

use crate::config::constants::{APP_RESOURCE_DIR, VSCODE_EXTENSIONS_FILENAME};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::Cmd;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait VSCodeOperations {
    fn export(&self) -> Result<(), AppError>;
    fn import(&self) -> Result<(), AppError>;
    fn ensure_code_command(&self) -> Result<(), AppError>;
}

pub struct VSCode {
    shell: Arc<dyn ShellOperations>,
}

impl VSCode {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    fn check_code_command(&self) -> Result<(), AppError> {
        self.shell.which(Cmd::Cmd("code".into()))?;
        Ok(())
    }

    fn list_extensions(&self) -> Result<Vec<u8>, AppError> {
        let output = self
            .shell
            .shell(Cmd::Cmd("code --list-extensions".into()))?;

        debug!("code --list-extensions output: {:?}", output);

        if !output.status.success() {
            return Err(AppError::VSCode("Failed to list extensions".to_string()));
        }
        Ok(output.stdout)
    }

    fn save_extensions(&self, extensions: &[u8]) -> Result<(), AppError> {
        let path = self.get_extensions_path();
        create_dir_all(path.parent().unwrap()).map_err(|e| AppError::Io(Arc::new(e)))?;
        let mut file = File::create(&path).map_err(|e| AppError::Io(Arc::new(e)))?;
        file.write_all(extensions)
            .map_err(|e| AppError::Io(Arc::new(e)))?;
        Ok(())
    }

    fn get_extensions_path(&self) -> PathBuf {
        Path::new(APP_RESOURCE_DIR).join(VSCODE_EXTENSIONS_FILENAME)
    }

    fn read_extensions(&self, path: &Path) -> Result<Vec<String>, AppError> {
        let file = File::open(path).map_err(|e| AppError::Io(Arc::new(e)))?;
        BufReader::new(file)
            .lines()
            .collect::<Result<_, _>>()
            .map_err(|e| AppError::Io(Arc::new(e)))
    }

    fn install_extension(&self, extension: &str) -> Result<(), AppError> {
        let output = self.shell.shell(Cmd::Cmd(
            format!("code --install-extension --force {}", extension).into(),
        ))?;

        debug!("code --install-extension --force output: {:?}", output);

        if !output.status.success() {
            return Err(AppError::VSCode(format!(
                "Failed to install extension: {}",
                extension
            )));
        }
        Ok(())
    }
}

impl VSCodeOperations for VSCode {
    fn export(&self) -> Result<(), AppError> {
        self.check_code_command()?;
        let extensions = self.list_extensions()?;
        self.save_extensions(&extensions)
    }

    fn import(&self) -> Result<(), AppError> {
        let path = self.get_extensions_path();
        let extensions = self.read_extensions(&path)?;
        for ext in extensions {
            self.install_extension(&ext)?;
        }
        Ok(())
    }

    fn ensure_code_command(&self) -> Result<(), AppError> {
        self.check_code_command()?;
        println!("Code command is already installed");
        Ok(())
    }
}
