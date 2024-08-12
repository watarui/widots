use log::debug;

use crate::config::constants::{
    APP_RESOURCE_DIR, BREW_CASK_FORMULA_FILENAME, BREW_FORMULA_FILENAME,
};
use crate::core::shell::ShellOperations;
use crate::error::app_error::AppError;
use crate::models::shell::Cmd;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;

pub trait HomebrewOperations {
    fn install(&self) -> Result<(), AppError>;
    fn import(&self) -> Result<(), AppError>;
    fn export(&self) -> Result<(), AppError>;
    fn is_installed(&self) -> bool;
}

pub struct Homebrew {
    shell: Arc<dyn ShellOperations>,
}

impl Homebrew {
    pub fn new(shell: Arc<dyn ShellOperations>) -> Self {
        Self { shell }
    }

    fn install_homebrew(&self) -> Result<(), AppError> {
        let install_script = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
        let output = self.shell.bash(install_script)?;

        debug!("Homebrew install output: {:?}", output);

        if output.status.success() {
            Ok(())
        } else {
            Err(AppError::Homebrew("Failed to install Homebrew".to_string()))
        }
    }

    fn import_formula(&self, formula_path: &PathBuf) -> Result<(), AppError> {
        debug!("Importing formula... from: {:?}", formula_path.display());

        self.install_formula(formula_path, "")
    }

    fn import_cask(&self, cask_path: &PathBuf) -> Result<(), AppError> {
        debug!("Importing cask formula... from: {:?}", cask_path.display());

        self.install_formula(cask_path, "--cask")
    }

    fn install_formula(&self, formula_path: &PathBuf, arg: &str) -> Result<(), AppError> {
        debug!("Install formula path: {:?}", formula_path.display());

        let file = File::open(formula_path).map_err(|e| AppError::Io(Arc::new(e)))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let formula = line.map_err(|e| AppError::Io(Arc::new(e)))?;

            debug!("Install formula: {:?}", formula);

            let output = self
                .shell
                .shell(Cmd::Cmd(format!("brew install {} {}", arg, formula).into()))?;

            debug!("Install formula output: {:?}", output);

            if !output.status.success() {
                println!("Failed to install formula: {}", formula);
            }
        }
        Ok(())
    }

    fn export_formula(&self, args: &[String], path: &PathBuf) -> Result<(), AppError> {
        let output = self
            .shell
            .shell(Cmd::Cmd(format!("brew {}", args.join(" ")).into()))?;

        debug!("Export formula output: {:?}", output);

        let mut file = File::create(path).map_err(|e| AppError::Io(Arc::new(e)))?;
        file.write_all(&output.stdout)
            .map_err(|e| AppError::Io(Arc::new(e)))?;
        Ok(())
    }
}

impl HomebrewOperations for Homebrew {
    fn install(&self) -> Result<(), AppError> {
        if self.is_installed() {
            return Err(AppError::Homebrew(
                "Homebrew is already installed".to_string(),
            ));
        }

        self.install_homebrew()
    }

    fn import(&self) -> Result<(), AppError> {
        if !self.is_installed() {
            return Err(AppError::Homebrew("Homebrew is not installed".to_string()));
        }

        let formula_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_FORMULA_FILENAME);
        let cask_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_CASK_FORMULA_FILENAME);

        self.import_formula(&formula_path)?;
        self.import_cask(&cask_path)?;

        Ok(())
    }

    fn export(&self) -> Result<(), AppError> {
        if !self.is_installed() {
            return Err(AppError::Homebrew("Homebrew is not installed".to_string()));
        }

        create_dir_all(APP_RESOURCE_DIR).map_err(|e| AppError::Io(Arc::new(e)))?;

        let formula_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_FORMULA_FILENAME);
        self.export_formula(&["leaves".to_string()], &formula_path)?;

        let cask_path = PathBuf::from(APP_RESOURCE_DIR).join(BREW_CASK_FORMULA_FILENAME);
        self.export_formula(
            &["list".to_string(), "--cask".to_string(), "-1".to_string()],
            &cask_path,
        )?;

        Ok(())
    }

    fn is_installed(&self) -> bool {
        self.shell.which(Cmd::Cmd("brew".into())).is_ok()
    }
}
