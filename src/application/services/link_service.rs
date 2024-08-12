use crate::domain::link::LinkOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use std::path::Path;
use std::sync::Arc;

pub struct LinkService {
    link_operations: Arc<dyn LinkOperations>,
}

impl LinkService {
    pub fn new(link_operations: Arc<dyn LinkOperations>) -> Self {
        Self { link_operations }
    }

    pub async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        self.link_operations
            .link_recursively(source, target, force)
            .await
    }

    pub async fn materialize_dotfiles(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        self.link_operations
            .materialize_symlinks_recursively(target)
            .await
    }
}
