use crate::domain::path::PathOperations;
use crate::error::AppError;
use async_trait::async_trait;
use dirs::home_dir;
use log::{debug, trace};
use std::path::{Path, PathBuf};

pub struct PathExpander;

impl PathExpander {
    pub fn new() -> Self {
        PathExpander
    }
}

#[async_trait]
impl PathOperations for PathExpander {
    async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = if path.starts_with("~") {
            self.get_home_dir()?.join(
                path.strip_prefix("~")
                    .map_err(|e| AppError::IoError(e.to_string()))?,
            )
        } else {
            path.to_path_buf()
        };
        Ok(expanded_path)
    }

    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = self.expand_tilde(path).await?;

        debug!("Parsed path: {:?}", expanded_path.display());
        trace!("Parsed path: {:?}", expanded_path.display());

        expanded_path
            .canonicalize()
            .map_err(|e| AppError::IoError(e.to_string()))
    }

    fn get_home_dir(&self) -> Result<PathBuf, AppError> {
        home_dir().ok_or(AppError::DirectoryNotFound)
    }
}
