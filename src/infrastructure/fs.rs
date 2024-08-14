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

pub struct FileSystemOperationsImpl;

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
