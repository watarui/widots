use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::error::app_error::AppError;

/// Provides asynchronous I/O operations.
#[async_trait]
pub trait IOOperations: Send + Sync {
    /// Reads lines from a file asynchronously.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file to read
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of strings (lines from the file),
    /// or an `AppError` if the operation fails.
    async fn read_lines(&self, file_path: &Path) -> Result<Vec<String>, AppError>;

    /// Writes lines to a file asynchronously.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file to write
    /// * `lines` - The lines to write to the file
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    async fn write_lines(&self, file_path: &Path, lines: &[String]) -> Result<(), AppError>;
}

/// Implements asynchronous I/O operations.
pub struct IO;

impl Default for IO {
    fn default() -> Self {
        Self::new()
    }
}

impl IO {
    /// Creates a new `IO` instance.
    pub fn new() -> Self {
        IO
    }
}

#[async_trait]
impl IOOperations for IO {
    async fn read_lines(&self, file_path: &Path) -> Result<Vec<String>, AppError> {
        let file = File::open(file_path).await.map_err(AppError::Io)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();

        let mut lines_stream = reader.lines();
        while let Some(line) = lines_stream.next_line().await.map_err(AppError::Io)? {
            lines.push(line);
        }

        if lines.is_empty() {
            Err(AppError::Unexpected("File is empty".to_string()))
        } else {
            Ok(lines)
        }
    }

    async fn write_lines(&self, file_path: &Path, lines: &[String]) -> Result<(), AppError> {
        let mut file = File::create(file_path).await.map_err(AppError::Io)?;
        for line in lines {
            file.write_all(line.as_bytes())
                .await
                .map_err(AppError::Io)?;
            file.write_all(b"\n").await.map_err(AppError::Io)?;
        }
        Ok(())
    }
}

/// Reads lines from a file asynchronously.
///
/// This is a convenience function that creates an `IO` instance and calls its `read_lines` method.
///
/// # Arguments
///
/// * `file_path` - The path to the file to read
///
/// # Returns
///
/// A `Result` containing a vector of strings (lines from the file),
/// or an `AppError` if the operation fails.
pub async fn read_lines(file_path: &Path) -> Result<Vec<String>, AppError> {
    IO::new().read_lines(file_path).await
}

/// Writes lines to a file asynchronously.
///
/// This is a convenience function that creates an `IO` instance and calls its `write_lines` method.
///
/// # Arguments
///
/// * `file_path` - The path to the file to write
/// * `lines` - The lines to write to the file
///
/// # Returns
///
/// A `Result` indicating success or failure of the operation.
pub async fn write_lines(file_path: &Path, lines: &[String]) -> Result<(), AppError> {
    IO::new().write_lines(file_path, lines).await
}
