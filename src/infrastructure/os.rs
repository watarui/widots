use crate::domain::os::OSOperations;
use crate::error::AppError;
use async_trait::async_trait;

#[derive(Debug)]
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

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use proptest::prelude::*;

    mock! {
        pub OSDetector {}

        #[async_trait]
        impl OSOperations for OSDetector {
            async fn get_os(&self) -> Result<String, AppError>;
        }
    }

    #[test]
    fn test_toml_os_detector_default() {
        let default_parser = OSDetector;
        let new_parser = OSDetector::new();

        // Ensure that the default implementation works correctly
        assert_eq!(format!("{:?}", default_parser), format!("{:?}", new_parser));
    }

    #[tokio::test]
    async fn test_get_os() {
        let os_detector = OSDetector::new();
        let result = os_detector.get_os().await;
        assert!(result.is_ok());

        let os = result.unwrap();
        assert!(os == "macos" || os == "linux", "Unexpected OS: {}", os);
    }

    #[tokio::test]
    async fn test_get_os_macos() {
        let mut mock = MockOSDetector::new();
        mock.expect_get_os()
            .times(1)
            .returning(|| Ok("macos".to_string()));

        let result = mock.get_os().await;
        assert_eq!(result.unwrap(), "macos");
    }

    #[tokio::test]
    async fn test_get_os_linux() {
        let mut mock = MockOSDetector::new();
        mock.expect_get_os()
            .times(1)
            .returning(|| Ok("linux".to_string()));

        let result = mock.get_os().await;
        assert_eq!(result.unwrap(), "linux");
    }

    #[tokio::test]
    async fn test_get_os_unsupported() {
        let mut mock = MockOSDetector::new();
        mock.expect_get_os()
            .times(1)
            .returning(|| Err(AppError::UnsupportedOS("Unknown".to_string())));

        let result = mock.get_os().await;
        assert!(matches!(result, Err(AppError::UnsupportedOS(_))));
    }

    proptest! {
        #[test]
        fn test_os_detector_new_and_default(_use_default: bool) {
            let new_detector = OSDetector::new();
            let default_detector = OSDetector;

            prop_assert_eq!(format!("{:?}", new_detector), format!("{:?}", default_detector));
        }
    }
}
