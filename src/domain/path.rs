use crate::error::AppError;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[async_trait]
pub trait PathOperations: Send + Sync {
    async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
    fn get_home_dir(&self) -> Result<PathBuf, AppError>;
}
