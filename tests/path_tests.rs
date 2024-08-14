use crate::domain::path::PathOperations;
use crate::error::AppError;
use crate::infrastructure::path::PathExpander;
use std::path::Path;

#[tokio::test]
async fn test_expand_tilde() -> Result<(), AppError> {
    let path_expander = PathExpander::new();
    let result = path_expander.expand_tilde(Path::new("~/test")).await?;
    assert!(result.to_str().unwrap().contains("/test"));
    assert!(!result.to_str().unwrap().contains("~"));
    Ok(())
}

#[tokio::test]
async fn test_parse_path() -> Result<(), AppError> {
    let path_expander = PathExpander::new();
    let result = path_expander.parse_path(Path::new("/tmp/test")).await?;
    assert_eq!(result, Path::new("/tmp/test"));
    Ok(())
}

#[tokio::test]
async fn test_get_home_dir() -> Result<(), AppError> {
    let path_expander = PathExpander::new();
    let result = path_expander.get_home_dir().await?;
    assert!(result.is_absolute());
    Ok(())
}
