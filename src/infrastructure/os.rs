use crate::domain::os::OSOperations;
use crate::error::AppError;
use async_trait::async_trait;

pub struct OSDetector;

impl Default for OSDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl OSDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OSOperations for OSDetector {
    async fn get_os(&self) -> Result<String, AppError> {
        #[cfg(target_os = "macos")]
        return Ok("macos".to_string());

        #[cfg(target_os = "linux")]
        return Ok("linux".to_string());

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        return Err(AppError::UnsupportedOS("Unknown".to_string()));
    }
}

#[tokio::test]
async fn test_get_os() {
    let os_detector = OSDetector::new();
    let result = os_detector.get_os().await;
    assert!(result.is_ok());

    let os = result.unwrap();
    assert!(os == "macos" || os == "linux", "Unexpected OS: {}", os);
}
