use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::error::AppError;
use crate::infrastructure::link::LinkerImpl;
use crate::models::link::FileProcessResult;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

struct MockPathOperations;

#[async_trait::async_trait]
impl PathOperations for MockPathOperations {
    async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError> {
        Ok(path.to_path_buf())
    }

    async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError> {
        Ok(path.to_path_buf())
    }

    async fn get_home_dir(&self) -> Result<PathBuf, AppError> {
        Ok(PathBuf::from("/home/user"))
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

    let path_ops = Arc::new(MockPathOperations);
    let linker = LinkerImpl::new(path_ops);

    let results = linker
        .link_recursively(&source_dir, &target_dir, false)
        .await?;

    assert_eq!(results.len(), 2);
    for result in results {
        match result {
            FileProcessResult::Linked(_, _) => {}
            _ => panic!("Expected Linked result"),
        }
    }

    assert!(fs::symlink_metadata(target_dir.join("file1.txt"))
        .await?
        .is_symlink());
    assert!(fs::symlink_metadata(target_dir.join("file2.txt"))
        .await?
        .is_symlink());

    Ok(())
}
