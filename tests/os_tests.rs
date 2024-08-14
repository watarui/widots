use crate::domain::os::OSOperations;
use crate::infrastructure::os::OSDetector;

#[tokio::test]
async fn test_get_os() {
    let os_detector = OSDetector::new();
    let result = os_detector.get_os().await;
    assert!(result.is_ok());

    let os = result.unwrap();
    assert!(os == "macos" || os == "linux", "Unexpected OS: {}", os);
}
