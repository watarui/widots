use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Arc;

use crate::error::app_error::AppError;

pub trait IOOperations {
    fn read_lines(&self, file_path: &Path) -> Result<Vec<String>, AppError>;
}

pub struct IO;

impl Default for IO {
    fn default() -> Self {
        IO::new()
    }
}

impl IO {
    pub fn new() -> Self {
        IO
    }
}

impl IOOperations for IO {
    fn read_lines(&self, file_path: &Path) -> Result<Vec<String>, AppError> {
        let file = File::open(file_path).map_err(|e| AppError::Io(Arc::new(e)))?;
        let reader = io::BufReader::new(file);

        let lines: Vec<String> = reader
            .lines()
            .collect::<Result<_, _>>()
            .map_err(|e| AppError::Io(Arc::new(e)))?;

        if lines.is_empty() {
            Err(AppError::Unexpected("File is empty".to_string()))
        } else {
            Ok(lines)
        }
    }
}

pub fn read_lines(file_path: &Path) -> Result<Vec<String>, AppError> {
    IO.read_lines(file_path)
}
