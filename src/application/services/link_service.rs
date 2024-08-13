use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use inquire::Confirm;
use std::path::Path;
use std::sync::Arc;

pub struct LinkService {
    link_operations: Arc<dyn LinkOperations>,
    path_operations: Arc<dyn PathOperations>,
}

impl LinkService {
    pub fn new(
        link_operations: Arc<dyn LinkOperations>,
        path_operations: Arc<dyn PathOperations>,
    ) -> Self {
        Self {
            link_operations,
            path_operations,
        }
    }

    pub async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let source = self.path_operations.parse_path(source).await?;
        let target = self.path_operations.parse_path(target).await?;

        if !self.confirm_action(&format!(
            "This will link files from {:?} to {:?}. Do you want to continue?",
            source.display(),
            target.display()
        ))? {
            return Ok(vec![]);
        }

        self.link_operations
            .link_recursively(&source, &target, force)
            .await
    }

    pub async fn materialize_dotfiles(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let target = self.path_operations.parse_path(target).await?;

        if !self.confirm_action(&format!(
            "This will materialize symlinks in {:?}. Do you want to continue?",
            target.display()
        ))? {
            return Ok(vec![]);
        }

        self.link_operations
            .materialize_symlinks_recursively(&target)
            .await
    }

    fn confirm_action(&self, message: &str) -> Result<bool, AppError> {
        Confirm::new(message)
            .with_default(false)
            .prompt()
            .map_err(|e| AppError::IoError(e.to_string()))
    }
}
