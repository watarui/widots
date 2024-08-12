use crate::error::app_error::AppError;
use dirs::home_dir;
use log::debug;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub trait PathOperations: Send + Sync {
    fn expand_home(&self, path: &Path) -> Result<PathBuf, AppError>;
    fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
}

pub struct PathExpander;

impl Default for PathExpander {
    fn default() -> Self {
        PathExpander::new()
    }
}

impl PathExpander {
    pub fn new() -> Self {
        PathExpander
    }
}

impl PathOperations for PathExpander {
    fn expand_home(&self, path: &Path) -> Result<PathBuf, AppError> {
        let mut path = path.to_path_buf();
        if path.starts_with("~") {
            let home = home_dir().ok_or(AppError::HomeDirectoryNotFound)?;
            path = home.join(
                path.strip_prefix("~")
                    .map_err(AppError::PathStripPrefixError)?,
            );
        }
        Ok(path)
    }

    fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = self.expand_home(path)?;

        debug!("Parsed path: {:?}", expanded_path.display());

        expanded_path
            .canonicalize()
            .map_err(|e| AppError::PathCanonicalizationError(Arc::new(e)))
    }
}
