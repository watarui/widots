use crate::commands::link::LinkOperations;
use crate::error::app_error::AppError;
use crate::models::link::FileProcessResult;
use std::path::Path;
use std::sync::Arc;

pub trait Materializable {
    fn execute(&self, destination: &Path) -> Result<(), AppError>;
}

pub struct Materializer {
    linker: Arc<dyn LinkOperations>,
}

impl Materializer {
    pub fn new(linker: Arc<dyn LinkOperations>) -> Self {
        Self { linker }
    }
}

impl Materializable for Materializer {
    fn execute(&self, destination: &Path) -> Result<(), AppError> {
        let results = self.linker.materialize_symlinks_recursively(destination)?;

        for result in results {
            match result {
                FileProcessResult::Materialized(symlink, target) => {
                    println!("Materialized symlink {:?} to {:?}", symlink, target);
                }
                FileProcessResult::Error(e) => {
                    println!("Error materializing symlink: {:?}", e);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
