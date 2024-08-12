use crate::config::constants::{LINK_IGNORED_ANCESTORS, LINK_IGNORED_FILES, LINK_IGNORED_PREFIXES};
use crate::domain::link::LinkOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::infrastructure::fs::FileSystemOperations;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

pub struct LinkerImpl {
    _fs_operations: Arc<dyn FileSystemOperations>,
    _shell_executor: Arc<dyn ShellExecutor>,
}

impl LinkerImpl {
    pub fn new(
        fs_operations: Arc<dyn FileSystemOperations>,
        shell_executor: Arc<dyn ShellExecutor>,
    ) -> Self {
        Self {
            _fs_operations: fs_operations,
            _shell_executor: shell_executor,
        }
    }

    async fn create_symlink(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<FileProcessResult, AppError> {
        if target.exists() {
            if force {
                fs::remove_file(target)
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;
            } else {
                return Ok(FileProcessResult::Skipped(target.to_path_buf()));
            }
        }

        fs::symlink(source, target)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;
        Ok(FileProcessResult::Linked(
            source.to_path_buf(),
            target.to_path_buf(),
        ))
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        LINK_IGNORED_FILES.contains(file_name)
            || LINK_IGNORED_PREFIXES
                .iter()
                .any(|&prefix| file_name.starts_with(prefix))
            || path.ancestors().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| LINK_IGNORED_ANCESTORS.contains(&name))
                    .unwrap_or(false)
            })
    }
}

#[async_trait]
impl LinkOperations for LinkerImpl {
    async fn link_recursively(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let mut results = Vec::new();
        let mut dirs = vec![source.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(&dir)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?
            {
                let path = entry.path();
                if self.should_ignore(&path) {
                    continue;
                }

                let relative = path
                    .strip_prefix(source)
                    .map_err(|e| AppError::PathError(e.to_string()))?;
                let target_path = target.join(relative);

                if path.is_dir() {
                    dirs.push(path.clone());
                    fs::create_dir_all(&target_path)
                        .await
                        .map_err(|e| AppError::IoError(e.to_string()))?;
                    results.push(FileProcessResult::Created(target_path));
                } else {
                    let result = self.create_symlink(&path, &target_path, force).await?;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    async fn materialize_symlinks_recursively(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let mut results = Vec::new();
        let mut dirs = vec![target.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(&dir)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?
            {
                let path = entry.path();

                if path.is_dir() {
                    dirs.push(path);
                } else if path.is_symlink() {
                    let target = fs::read_link(&path)
                        .await
                        .map_err(|e| AppError::IoError(e.to_string()))?;
                    fs::remove_file(&path)
                        .await
                        .map_err(|e| AppError::IoError(e.to_string()))?;
                    fs::copy(&target, &path)
                        .await
                        .map_err(|e| AppError::IoError(e.to_string()))?;
                    results.push(FileProcessResult::Materialized(path, target));
                }
            }
        }

        Ok(results)
    }
}
