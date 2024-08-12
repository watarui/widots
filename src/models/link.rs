use crate::error::AppError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FileProcessResult {
    Linked(PathBuf, PathBuf),
    Created(PathBuf),
    Materialized(PathBuf, PathBuf),
    Skipped(PathBuf),
    _Error(AppError),
}
