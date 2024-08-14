use crate::config::constants::{LINK_IGNORED_ANCESTORS, LINK_IGNORED_FILES, LINK_IGNORED_PREFIXES};
use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

pub struct LinkerImpl {
    path_operations: Arc<dyn PathOperations>,
}

impl LinkerImpl {
    pub fn new(path_operations: Arc<dyn PathOperations>) -> Self {
        Self { path_operations }
    }

    async fn create_symlink(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<FileProcessResult, AppError> {
        if target.exists() {
            if force {
                fs::remove_file(target).await?;
            } else {
                return Ok(FileProcessResult::Skipped(target.to_path_buf()));
            }
        }

        fs::symlink(source, target).await?;

        Ok(FileProcessResult::Linked(
            source.to_path_buf(),
            target.to_path_buf(),
        ))
    }

    async fn ensure_parent_directory(&self, path: &Path) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    async fn process_entry(
        &self,
        entry: walkdir::DirEntry,
        source_path: &Path,
        target_path: &Path,
        force: bool,
    ) -> FileProcessResult {
        let path = entry.path();
        let relative_path = path.strip_prefix(source_path).unwrap();
        let target = target_path.join(relative_path);

        if path.is_file() {
            match self.ensure_parent_directory(&target).await {
                Ok(_) => self
                    .create_symlink(path, &target, force)
                    .await
                    .unwrap_or_else(FileProcessResult::Error),
                Err(e) => FileProcessResult::Error(e),
            }
        } else if path.is_dir() {
            if !target.exists() {
                fs::create_dir_all(&target)
                    .await
                    .map(|_| FileProcessResult::Created(target))
                    .unwrap_or_else(|e| FileProcessResult::Error(AppError::Io(e)))
            } else {
                FileProcessResult::Skipped(target)
            }
        } else {
            FileProcessResult::Skipped(path.to_path_buf())
        }
    }

    async fn materialize_symlink(&self, path: &Path) -> Result<FileProcessResult, AppError> {
        let target = fs::read_link(path).await?;
        fs::remove_file(path).await?;
        fs::copy(&target, path).await?;
        Ok(FileProcessResult::Materialized(path.to_path_buf(), target))
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
        let walker = walkdir::WalkDir::new(source).into_iter();
        let filtered_entries = walker
            .filter_entry(|e| !self.should_ignore(e.path()))
            .filter_map(Result::ok);

        let results = stream::iter(filtered_entries)
            .then(|entry| async move { self.process_entry(entry, source, target, force).await })
            .collect::<Vec<_>>()
            .await;

        Ok(results)
    }

    async fn materialize_symlinks_recursively(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let mut results = Vec::new();
        let mut dirs = vec![target.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(&dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    dirs.push(path);
                } else if path.is_symlink() {
                    results.push(self.materialize_symlink(&path).await?);
                }
            }
        }

        Ok(results)
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        LINK_IGNORED_FILES.contains(file_name)
            || LINK_IGNORED_PREFIXES
                .iter()
                .any(|prefix| file_name.starts_with(prefix))
            || path.ancestors().any(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| LINK_IGNORED_ANCESTORS.contains(name))
                    .unwrap_or(false)
            })
    }
}
