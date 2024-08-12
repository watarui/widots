use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Represents a file or directory in the file system.
#[derive(Debug, Clone)]
pub struct File {
    /// The path to the file or directory.
    pub path: PathBuf,
    /// Whether this is a directory.
    pub is_directory: bool,
    /// The last modification time of the file or directory.
    pub last_modified: Option<DateTime<Utc>>,
    /// The size of the file in bytes.
    pub size: u64,
}

impl File {
    /// Creates a new `File` instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file or directory
    /// * `is_directory` - Whether this is a directory
    /// * `last_modified` - The last modification time
    /// * `size` - The size of the file in bytes
    ///
    /// # Returns
    ///
    /// A new `File` instance.
    pub fn new(
        path: PathBuf,
        is_directory: bool,
        last_modified: Option<DateTime<Utc>>,
        size: u64,
    ) -> Self {
        Self {
            path,
            is_directory,
            last_modified,
            size,
        }
    }
}
