use crate::error::AppError;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[async_trait]
pub trait FileSystemOperations: Send + Sync {
    async fn read_lines(&self, path: &Path) -> Result<Vec<String>, AppError>;
    async fn write_lines(&self, path: &Path, lines: &[String]) -> Result<(), AppError>;
}

#[derive(Debug)]
pub struct FileSystemOperationsImpl;

impl Default for FileSystemOperationsImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemOperationsImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystemOperations for FileSystemOperationsImpl {
    async fn read_lines(&self, path: &Path) -> Result<Vec<String>, AppError> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut lines_stream = reader.lines();

        while let Some(line) = lines_stream.next_line().await? {
            lines.push(line);
        }

        Ok(lines)
    }

    async fn write_lines(&self, path: &Path, lines: &[String]) -> Result<(), AppError> {
        let mut file = File::create(path).await?;
        for line in lines {
            file.write_all(line.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::AppError;
    use mockall::predicate::*;
    use mockall::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    mock! {
        pub FileSystemOperationsImpl {}

        #[async_trait]
        impl FileSystemOperations for FileSystemOperationsImpl {
            async fn read_lines(&self, path: &Path) -> Result<Vec<String>, AppError>;
            async fn write_lines(&self, path: &Path, lines: &[String]) -> Result<(), AppError>;
        }
    }

    #[tokio::test]
    async fn test_read_lines_with_mock() {
        let mut mock = MockFileSystemOperationsImpl::new();
        mock.expect_read_lines()
            .with(eq(Path::new("/test/file.txt")))
            .times(1)
            .returning(|_| Ok(vec!["Line 1".to_string(), "Line 2".to_string()]));

        let result = mock.read_lines(Path::new("/test/file.txt")).await.unwrap();
        assert_eq!(result, vec!["Line 1".to_string(), "Line 2".to_string()]);
    }

    #[tokio::test]
    async fn test_write_lines_with_mock() {
        let mut mock = MockFileSystemOperationsImpl::new();
        mock.expect_write_lines()
            .with(
                eq(Path::new("/test/file.txt")),
                eq(["Line 1".to_string(), "Line 2".to_string()]),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let result = mock
            .write_lines(
                Path::new("/test/file.txt"),
                &["Line 1".to_string(), "Line 2".to_string()],
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_write_lines_with_temp_file() -> Result<(), AppError> {
        let temp_dir = TempDir::new().unwrap();
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

    #[tokio::test]
    async fn test_read_lines_non_existent_file() {
        let fs_ops = FileSystemOperationsImpl::new();
        let result = fs_ops.read_lines(Path::new("/non/existent/file.txt")).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_file_system_operations_impl_default() {
        let default_parser = FileSystemOperationsImpl;
        let new_parser = FileSystemOperationsImpl::new();

        // Ensure that the default implementation works correctly
        assert_eq!(format!("{:?}", default_parser), format!("{:?}", new_parser));
    }

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

    proptest! {
        #[test]
        fn test_write_read_lines_roundtrip(lines in prop::collection::vec(String::arbitrary(), 0..100)) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let file_path = temp_dir.path().join("test.txt");
                let fs_ops = FileSystemOperationsImpl::new();

                fs_ops.write_lines(&file_path, &lines).await.unwrap();
                let read_lines = fs_ops.read_lines(&file_path).await.unwrap();

                prop_assert_eq!(lines, read_lines);
                Ok(())
            }).unwrap();
        }

        #[test]
        fn test_write_lines_non_empty(lines in prop::collection::vec(String::arbitrary(), 1..100)) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ =rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let file_path = temp_dir.path().join("test.txt");
                let fs_ops = FileSystemOperationsImpl::new();

                fs_ops.write_lines(&file_path, &lines).await.unwrap();
                let metadata = tokio::fs::metadata(&file_path).await.unwrap();

                prop_assert!(metadata.len() > 0);
                Ok(())
            });
        }
    }
}
