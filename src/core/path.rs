use crate::error::app_error::AppError;
use async_trait::async_trait;
use dirs::home_dir;
use log::debug;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Provides operations for path manipulation and expansion.
#[async_trait]
pub trait PathOperations: Send + Sync {
    /// Expands the tilde (~) in a path to the user's home directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to expand
    ///
    /// # Returns
    ///
    /// The expanded path, or an error if the home directory couldn't be determined.
    async fn expand_home(&self, path: &Path) -> Result<PathBuf, AppError>;

    /// Parses and expands a path, resolving any environment variables or tilde notation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to parse and expand
    ///
    /// # Returns
    ///
    /// The parsed and expanded path, or an error if the path couldn't be resolved.
    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
}

/// Expands and manipulates file system paths.
pub struct PathExpander;

impl Default for PathExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl PathExpander {
    /// Creates a new `PathExpander`.
    pub fn new() -> Self {
        PathExpander
    }
}

#[async_trait]
impl PathOperations for PathExpander {
    async fn expand_home(&self, path: &Path) -> Result<PathBuf, AppError> {
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

    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError> {
        let expanded_path = self.expand_home(path).await?;

        debug!("Parsed path: {:?}", expanded_path.display());

        fs::canonicalize(&expanded_path)
            .await
            .map_err(|e| AppError::PathCanonicalizationError(e.to_string()))
    }
}
