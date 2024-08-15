use crate::constants::{LINK_IGNORED_ANCESTORS, LINK_IGNORED_FILES, LINK_IGNORED_PREFIXES};
use crate::domain::link::LinkOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use regex::Regex;
use std::path::Path;
#[cfg(test)]
use tempfile::TempDir;
use tokio::fs;

pub struct LinkerImpl;

impl Default for LinkerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkerImpl {
    pub fn new() -> Self {
        Self
    }

    fn validate_filename(&self, filename: &str) -> Result<(), String> {
        if filename.is_empty() {
            return Err("Filename cannot be empty".to_string());
        }

        if filename == "." || filename == ".." {
            return Err("Filename cannot be '.' or '..'".to_string());
        }

        let valid_chars = Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
        if !valid_chars.is_match(filename) {
            return Err("Filename contains invalid characters".to_string());
        }

        Ok(())
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

        Ok(match fs::symlink(source, target).await {
            Ok(_) => FileProcessResult::Linked(source.to_path_buf(), target.to_path_buf()),
            Err(e) => FileProcessResult::Error(AppError::Symlink(e.to_string())),
        })
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
        let mut results = Vec::new();

        if !target.exists() {
            match fs::create_dir_all(target).await {
                Ok(_) => results.push(FileProcessResult::Created(target.to_path_buf())),
                Err(e) => return Err(AppError::Io(e)),
            }
        }

        let mut entries = fs::read_dir(source).await?;
        while let Some(entry) = entries.next_entry().await? {
            let ent = entry.path();

            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().ok_or_else(|| {
                AppError::InvalidFilename(file_name.to_string_lossy().to_string())
            })?;

            if let Err(e) = self.validate_filename(file_name_str) {
                println!("Skipped invalid file name: {} due to {}", file_name_str, e);
                continue;
            }

            if self.should_ignore(&ent) {
                println!("Ignored: {:?}", ent);
                continue;
            }

            let target = target.join(entry.file_name());

            if ent.is_dir() {
                match self.link_recursively(&ent, &target, force).await {
                    Ok(mut sub_results) => results.append(&mut sub_results),
                    Err(e) => return Err(e),
                }
            } else {
                match self.create_symlink(&ent, &target, force).await {
                    Ok(result) => results.push(result),
                    Err(e) => return Err(e),
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

#[tokio::test]
async fn test_link_recursively() -> Result<(), AppError> {
    let temp_dir = TempDir::new()?;
    let source_dir = temp_dir.path().join("source");
    let target_dir = temp_dir.path().join("target");

    fs::create_dir(&source_dir).await?;
    fs::create_dir(&target_dir).await?;

    fs::write(source_dir.join("file1.txt"), "content1").await?;
    fs::write(source_dir.join("file2.txt"), "content2").await?;
    let dir1_path = source_dir.join("dir1");
    if !dir1_path.exists() {
        fs::create_dir_all(&dir1_path).await?;
    }
    fs::write(dir1_path.join("file3.txt"), "content3").await?;

    let linker = LinkerImpl::new();

    let results = linker
        .link_recursively(&source_dir, &target_dir, false)
        .await?;

    assert_eq!(results.len(), 4);
    for result in results {
        match result {
            FileProcessResult::Linked(_, _) => {}
            FileProcessResult::Created(_) => {}
            _ => panic!("Expected Linked result"),
        }
    }

    assert!(fs::symlink_metadata(target_dir.join("file1.txt"))
        .await?
        .is_symlink());
    assert!(fs::symlink_metadata(target_dir.join("file2.txt"))
        .await?
        .is_symlink());
    assert!(fs::symlink_metadata(target_dir.join("dir1/file3.txt"))
        .await?
        .is_symlink());

    Ok(())
}
