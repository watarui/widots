use crate::error::AppError;
use crate::infrastructure::fs::{FileSystemOperations, FileSystemOperationsImpl};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_read_write_lines() -> Result<(), AppError> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");

    let fs_ops = FileSystemOperationsImpl::new();

    let lines_to_write = vec![
        "Line 1".to_string(),
        "Line 2".to_string(),
        "Line 3".to_string(),
    ];

    fs_ops.write_lines(&file_path, &lines_to_write).await?;

    let read_lines = fs_ops.read_lines(&file_path).await?;

    assert_eq!(lines_to_write, read_lines);

    Ok(())
}
