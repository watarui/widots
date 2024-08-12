use crate::error::app_error::AppError;

pub trait OSOperations: Send + Sync {
    fn get_os(&self) -> Result<String, AppError>;
}

pub struct OSDetector;

impl Default for OSDetector {
    fn default() -> Self {
        OSDetector::new()
    }
}

impl OSDetector {
    pub fn new() -> Self {
        OSDetector
    }
}

impl OSOperations for OSDetector {
    fn get_os(&self) -> Result<String, AppError> {
        #[cfg(target_os = "macos")]
        return Ok("mac".to_string());

        #[cfg(target_os = "linux")]
        return Ok("linux".to_string());

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        return Ok("unknown".to_string());
    }
}
