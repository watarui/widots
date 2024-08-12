use crate::error::app_error::AppError;
use std::path::PathBuf;

/// Represents the result of a file processing operation.
#[derive(Debug)]
pub enum FileProcessResult {
    /// A symlink was successfully created.
    Linked(PathBuf, PathBuf),
    /// A directory was created.
    Created(PathBuf),
    /// A symlink was materialized into a regular file.
    Materialized(PathBuf, PathBuf),
    /// The file was skipped.
    Skipped(PathBuf),
    /// An error occurred during processing.
    Error(AppError),
}
