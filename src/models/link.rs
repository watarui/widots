use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileProcessResult {
    Linked(PathBuf, PathBuf),
    Created(PathBuf),
    Materialized(PathBuf, PathBuf),
    Skipped(PathBuf),
}
