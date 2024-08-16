use crate::domain::link::LinkOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

#[cfg(test)]
use mockall::mock;
#[cfg(test)]
use prop::string::string_regex;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use tempfile::TempDir;

#[async_trait]
pub trait LinkService: Send + Sync {
    async fn link_dotfiles(
        &self,
        source: &Path,
        target: &Path,
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

        let mut results = self
            .link_operations
            .link_recursively(&source, &target)
            .await?;

        // Add existing target files to results if they're not already included
        let mut target_entries = tokio::fs::read_dir(target).await?;
        while let Some(entry) = target_entries.next_entry().await? {
            let path = entry.path();
            if !results.iter().any(|r| match r {
                FileProcessResult::Linked(_, dst) | FileProcessResult::Skipped(dst) => dst == &path,
                _ => false,
            }) {
                results.push(FileProcessResult::Skipped(path));
            }
        }

        Ok(results)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_provider::{ServiceProvider, TestServiceProvider};

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

        mock_link_ops.expect_link_recursively().returning(|_, _| {
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

        // Mock the tokio::fs::read_dir function
        tokio::task::LocalSet::new()
            .run_until(async {
                tokio::task::spawn_local(async {
                    tokio::fs::read_dir("/target")
                        .await
                        .unwrap()
                        .next_entry()
                        .await
                        .unwrap();
                })
                .await
                .unwrap();

                let result = link_service
                    .link_dotfiles(Path::new("/source"), Path::new("/target"))
                    .await;

                assert!(result.is_ok());
                let file_results = result.unwrap();
                assert_eq!(file_results.len(), 2);
            })
            .await;
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

    fn file_name_strategy() -> impl Strategy<Value = String> {
        prop::bool::ANY.prop_flat_map(|has_dot| {
            string_regex("[a-zA-Z][a-zA-Z0-9_]{0,9}")
                .unwrap()
                .prop_map(move |s| {
                    let s = if has_dot { format!(".{}", s) } else { s };
                    if rand::random() {
                        s.to_lowercase()
                    } else {
                        s.to_uppercase()
                    }
                })
        })
    }

    proptest! {
        #[test]
        fn link_service_doesnt_crash(
            source_files in prop::collection::vec(file_name_strategy(), 0..10),
            target_files in prop::collection::vec(file_name_strategy(), 0..5),
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
                let result = services.link_service().link_dotfiles(&source_path, &target_path).await;

                match result {
                    Ok(file_results) => {
                      // Check source files
                      for file in &source_files {
                          let file_lower = file.to_lowercase();
                          let file_result = file_results.iter().find(|r| {
                              match r {
                                  FileProcessResult::Linked(src, _) | FileProcessResult::Skipped(src) => {
                                      src.file_name().unwrap().to_str().unwrap().to_lowercase() == file_lower
                                  },
                                  _ => false
                              }
                          });
                          prop_assert!(file_result.is_some(), "Source file {} not found in results", file);
                      }

                      // Check that all target files are either linked or skipped
                      for file in &target_files {
                          let file_lower = file.to_lowercase();
                          let file_result = file_results.iter().find(|r| {
                              match r {
                                  FileProcessResult::Linked(_, dst) | FileProcessResult::Skipped(dst) => {
                                      dst.file_name().unwrap().to_str().unwrap().to_lowercase() == file_lower
                                  },
                                  _ => false
                              }
                          });
                          prop_assert!(file_result.is_some(), "Target file {} not found in results", file);
                      }
                    },
                    Err(e) => {
                        // Check if the error is expected
                        prop_assert!(matches!(e, AppError::Io(_) | AppError::Symlink(_)), "Unexpected error: {:?}", e);
                    }
                }

                Ok(())
            }).unwrap()
        }
    }
}
