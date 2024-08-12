use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub is_directory: bool,
}

impl File {
    pub fn new(path: PathBuf, is_directory: bool) -> Self {
        Self { path, is_directory }
    }
}
