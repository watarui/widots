use crate::error::app_error::AppError;
use async_trait::async_trait;

/// Provides operations for detecting and interacting with the operating system.
#[async_trait]
pub trait OSOperations: Send + Sync {
    /// Gets the current operating system.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` representing the operating system,
    /// or an `AppError` if the detection fails.
    async fn get_os(&self) -> Result<String, AppError>;
}

/// Detects the current operating system.
pub struct OSDetector;

impl Default for OSDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl OSDetector {
    /// Creates a new `OSDetector`.
    pub fn new() -> Self {
        OSDetector
    }
}

#[async_trait]
impl OSOperations for OSDetector {
    async fn get_os(&self) -> Result<String, AppError> {
        #[cfg(target_os = "macos")]
        return Ok("mac".to_string());

        #[cfg(target_os = "linux")]
        return Ok("linux".to_string());

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        return Ok("unknown".to_string());
    }
}
