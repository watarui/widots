use crate::error::app_error::AppError;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileProcessResult {
    Linked(PathBuf, PathBuf), // (source path, target path)
    Created(PathBuf),
    Materialized(PathBuf, PathBuf), // (symlink path, original target path)
    Skipped(PathBuf),
    Error(AppError),
}
