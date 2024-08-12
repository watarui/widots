use crate::commands::link::LinkOperations;
use crate::error::app_error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait Materializable: Send + Sync {
    async fn execute(&self, destination: &Path) -> Result<(), AppError>;
}

pub struct Materializer {
    linker: Arc<dyn LinkOperations>,
}

impl Materializer {
    pub fn new(linker: Arc<dyn LinkOperations>) -> Self {
        Self { linker }
    }
}

#[async_trait]
impl Materializable for Materializer {
    async fn execute(&self, destination: &Path) -> Result<(), AppError> {
        let results = self
            .linker
            .materialize_symlinks_recursively(destination)
            .await?;

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
