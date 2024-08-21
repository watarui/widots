use std::process::Output;

use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait ShellExecutor: Send + Sync {
    async fn execute<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<String, AppError>;
    async fn output<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<Output, AppError>;
    fn stderr(&self, output: &Output) -> String;
}
