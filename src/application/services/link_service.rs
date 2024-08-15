#[cfg(test)]
use crate::application::service_provider::ServiceProvider;
#[cfg(test)]
use crate::application::service_provider::TestServiceProvider;
use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
#[cfg(test)]
use mockall::mock;
#[cfg(test)]
use prop::string::string_regex;
#[cfg(test)]
use proptest::prelude::*;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(test)]
use tempfile::TempDir;

#[async_trait]
pub trait LinkService: Send + Sync {
    async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError>;
    async fn materialize_dotfiles(&self, target: &Path)
        -> Result<Vec<FileProcessResult>, AppError>;
}

pub struct LinkServiceImpl {
    link_operations: Arc<dyn LinkOperations>,
    path_operations: Arc<dyn PathOperations>,
    prompter: Arc<dyn PromptOperations>,
}

impl LinkServiceImpl {
    pub fn new(
        link_operations: Arc<dyn LinkOperations>,
        path_operations: Arc<dyn PathOperations>,
        prompter: Arc<dyn PromptOperations>,
    ) -> Self {
        Self {
            link_operations,
            path_operations,
            prompter,
        }
    }
}

#[async_trait]
impl LinkService for LinkServiceImpl {
    async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let source = self.path_operations.parse_path(source).await?;
        let target = self.path_operations.parse_path(target).await?;

        let ans = self
            .prompter
            .confirm_action(&format!(
                "This will link files from {:?} to {:?}. Do you want to continue?",
                source.display(),
                target.display()
            ))
            .await?;
        if !ans {
            return Ok(vec![]);
        }

        self.link_operations
            .link_recursively(&source, &target, force)
            .await
    }

    async fn materialize_dotfiles(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let target = self.path_operations.parse_path(target).await?;

        if !self
            .prompter
            .confirm_action(&format!(
                "This will materialize symlinks in {:?}. Do you want to continue?",
                target.display()
            ))
            .await?
        {
            return Ok(vec![]);
        }

        self.link_operations
            .materialize_symlinks_recursively(&target)
            .await
    }
}

#[cfg(test)]
mock! {
    LinkOperations {}
    #[async_trait]
    impl LinkOperations for LinkOperations {
        async fn link_recursively(
            &self,
            source: &Path,
            target: &Path,
            force: bool,
        ) -> Result<Vec<FileProcessResult>, AppError>;
        async fn materialize_symlinks_recursively(
            &self,
            target: &Path,
        ) -> Result<Vec<FileProcessResult>, AppError>;
        fn should_ignore(&self, path: &Path) -> bool;
    }
}

#[cfg(test)]
mock! {
    PathOperations {}
    #[async_trait]
    impl PathOperations for PathOperations {
        async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
        async fn get_home_dir(&self) -> Result<PathBuf, AppError>;
    }
}

#[cfg(test)]
mock! {
    PromptOperations {}
    #[async_trait]
    impl PromptOperations for PromptOperations {
        async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
    }
}

#[tokio::test]
async fn test_link_dotfiles() {
    let mut mock_link_ops = MockLinkOperations::new();
    let mut mock_path_ops = MockPathOperations::new();
    let mut mock_prompt_ops = MockPromptOperations::new();

    mock_path_ops
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    mock_prompt_ops
        .expect_confirm_action()
        .returning(|_| Ok(true));

    mock_link_ops
        .expect_link_recursively()
        .returning(|_, _, _| {
            Ok(vec![
                FileProcessResult::Linked(
                    PathBuf::from("/source/file1"),
                    PathBuf::from("/target/file1"),
                ),
                FileProcessResult::Created(PathBuf::from("/target/dir1")),
            ])
        });

    let link_service = LinkServiceImpl::new(
        Arc::new(mock_link_ops),
        Arc::new(mock_path_ops),
        Arc::new(mock_prompt_ops),
    );

    let result = link_service
        .link_dotfiles(Path::new("/source"), Path::new("/target"), false)
        .await;

    assert!(result.is_ok());
    let file_results = result.unwrap();
    assert_eq!(file_results.len(), 2);
}

#[tokio::test]
async fn test_materialize_dotfiles() {
    let mut mock_link_ops = MockLinkOperations::new();
    let mut mock_path_ops = MockPathOperations::new();
    let mut mock_prompt_ops = MockPromptOperations::new();

    mock_path_ops
        .expect_parse_path()
        .returning(|path| Ok(path.to_path_buf()));

    mock_prompt_ops
        .expect_confirm_action()
        .returning(|_| Ok(true));

    mock_link_ops
        .expect_materialize_symlinks_recursively()
        .returning(|_| {
            Ok(vec![FileProcessResult::Materialized(
                PathBuf::from("/target/file1"),
                PathBuf::from("/source/file1"),
            )])
        });

    let link_service = LinkServiceImpl::new(
        Arc::new(mock_link_ops),
        Arc::new(mock_path_ops),
        Arc::new(mock_prompt_ops),
    );

    let result = link_service
        .materialize_dotfiles(Path::new("/target"))
        .await;

    assert!(result.is_ok());
    let file_results = result.unwrap();
    assert_eq!(file_results.len(), 1);
}

#[cfg(test)]
fn file_name_strategy() -> impl Strategy<Value = String> {
    // Add dot to the beginning of the file name or not
    prop::bool::ANY.prop_flat_map(|has_dot| {
        string_regex("[a-zA-Z0-9_]{1,10}")
            .unwrap()
            .prop_map(move |s| if has_dot { format!(".{}", s) } else { s })
    })
}

#[cfg(test)]
proptest! {
    #[test]
    fn link_service_doesnt_crash(
        source_files in prop::collection::vec(file_name_strategy(), 0..10),
        target_files in prop::collection::vec(file_name_strategy(), 0..5),
        force in prop::bool::ANY
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let source_path = temp_dir.path().join("source");
            let target_path = temp_dir.path().join("target");
            tokio::fs::create_dir_all(&source_path).await.unwrap();
            tokio::fs::create_dir_all(&target_path).await.unwrap();

            // Create source files
            for file in &source_files {
                tokio::fs::write(source_path.join(file), "test content").await.unwrap();
            }

            // Create target files (simulate existing files)
            for file in &target_files {
                tokio::fs::write(target_path.join(file), "existing content").await.unwrap();
            }

            let services = TestServiceProvider::new(true);
            let result = services.link_service().link_dotfiles(&source_path, &target_path, force).await;

            match result {
                Ok(file_results) => {
                    for file in &source_files {
                        let file_in_target = target_files.contains(file);
                        let file_result = file_results.iter().find(|r| {
                            match r {
                                FileProcessResult::Linked(src, _) | FileProcessResult::Skipped(src) => {
                                    src.file_name().unwrap().to_str().unwrap() == file
                                },
                                _ => false
                            }
                        });

                        if file_result.is_none() {
                            println!("Warning: File {} not found in results", file);
                            continue;
                        }

                        match file_result.unwrap() {
                            FileProcessResult::Linked(_, _) => {
                                assert!(force || !file_in_target, "File {} should not be linked without force flag", file);
                            },
                            FileProcessResult::Skipped(_) => {
                                assert!(!force && file_in_target, "File {} should not be skipped with force flag or if not in target", file);
                            },
                            _ => println!("Unexpected result for file {}", file),
                        }
                    }

                    // Assert that all target files are linked or skipped
                    for file in &target_files {
                        let file_result = file_results.iter().find(|r| {
                            match r {
                                FileProcessResult::Linked(_, dst) | FileProcessResult::Skipped(dst) => {
                                    dst.file_name().unwrap().to_str().unwrap() == file
                                },
                                _ => false
                            }
                        });

                        if file_result.is_none() {
                            println!("Warning: Target file {} not found in results", file);
                            continue;
                        }

                        match file_result.unwrap() {
                            FileProcessResult::Linked(_, _) => {
                                assert!(force, "Target file {} should not be linked without force flag", file);
                            },
                            FileProcessResult::Skipped(_) => {
                                assert!(!force, "Target file {} should not be skipped with force flag", file);
                            },
                            _ => println!("Unexpected result for target file {}", file),
                        }
                    }
                },
                Err(e) => {
                    // Check if the error is expected
                    assert!(matches!(e, AppError::Io(_) | AppError::Symlink(_)));
                }
            }
        });
    }
}
