use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait LinkOperations: Send + Sync {
    async fn link_recursively(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError>;

    async fn materialize_symlinks_recursively(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError>;

    fn should_ignore(&self, path: &Path) -> bool;
}
