use crate::config::constants::{
    LINK_IGNORED_ANCESTORS, LINK_IGNORED_FILES, LINK_IGNORED_GIT_FILES, LINK_IGNORED_PREFIXES,
    LINK_TEST_DIR,
};
use crate::core::path::PathOperations;
use crate::error::app_error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use dirs::home_dir;
use inquire::Confirm;
use log::debug;
use std::fs::read_link;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;

#[async_trait]
pub trait LinkOperations: Send + Sync {
    async fn link_recursively(
        &self,
        source_path: &Path,
        target_path: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError>;
    async fn materialize_symlinks_recursively(
        &self,
        target_path: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError>;
    async fn confirmation(&self, message: &str, help_message: &str) -> Result<bool, AppError>;
}

pub struct Linker {
    path_expander: Arc<dyn PathOperations>,
}

impl Linker {
    pub fn new(path_expander: Arc<dyn PathOperations>) -> Arc<Self> {
        Arc::new(Self { path_expander })
    }

    fn should_ignore(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                LINK_IGNORED_FILES.iter().any(|&ignored| name == ignored)
                    || LINK_IGNORED_PREFIXES
                        .iter()
                        .any(|&prefix| name.starts_with(prefix))
                    || (path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|name| name.to_str())
                        .map(|parent_name| parent_name == "git")
                        .unwrap_or(false)
                        && LINK_IGNORED_GIT_FILES.contains(&name))
            })
            .unwrap_or(false)
            || path.ancestors().any(|p| {
                p.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| LINK_IGNORED_ANCESTORS.contains(&name))
                    .unwrap_or(false)
            })
    }

    async fn symlink_process(
        &self,
        entry: walkdir::DirEntry,
        source_path: &Path,
        target_path: &Path,
        force: bool,
    ) -> FileProcessResult {
        let path = entry.path();
        let relative_path = path.strip_prefix(source_path).unwrap();
        let target = target_path.join(relative_path);

        debug!("Link processing: {:?}", path.display());

        if path.is_file() {
            self.make_symlink(path, &target, force).await
        } else if path.is_dir() {
            if !target.exists() {
                debug!("Creating directory: {:?}", target);

                self.create_directory(&target).await
            } else {
                FileProcessResult::Skipped(target)
            }
        } else {
            FileProcessResult::Skipped(path.to_path_buf())
        }
    }

    async fn make_symlink(&self, path: &Path, target: &Path, force: bool) -> FileProcessResult {
        if let Err(e) = self.ensure_parent_directory(target).await {
            return FileProcessResult::Error(e);
        }

        if target.exists() {
            if force {
                if let Err(e) = fs::remove_file(target).await {
                    return FileProcessResult::Error(AppError::Io(e));
                }
            } else {
                return FileProcessResult::Skipped(target.to_path_buf());
            }
        }

        match symlink(path, target) {
            Ok(_) => {
                println!(
                    "Symlink created: {:?} -> {:?}",
                    path.display(),
                    target.display()
                );
                FileProcessResult::Linked(path.to_path_buf(), target.to_path_buf())
            }
            Err(e) => {
                FileProcessResult::Error(AppError::Link(format!("Failed to create symlink: {}", e)))
            }
        }
    }

    async fn create_directory(&self, target: &Path) -> FileProcessResult {
        match fs::create_dir_all(target).await {
            Ok(_) => FileProcessResult::Created(target.to_path_buf()),
            Err(e) => FileProcessResult::Error(AppError::Io(e)),
        }
    }

    async fn ensure_parent_directory(&self, path: &Path) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(AppError::Io)?;
        }
        Ok(())
    }

    async fn materialize_symlink(&self, symlink_path: &Path) -> FileProcessResult {
        debug!("Materializing symlink: {:?}", symlink_path.display());

        match read_link(symlink_path) {
            Ok(target_path) => {
                if let Err(e) = fs::remove_file(symlink_path).await {
                    return FileProcessResult::Error(AppError::Io(e));
                }

                match fs::copy(&target_path, symlink_path).await {
                    Ok(_) => {
                        println!(
                            "Symlink materialized: {:?} -> {:?}",
                            symlink_path.display(),
                            target_path.display()
                        );
                        FileProcessResult::Materialized(symlink_path.to_path_buf(), target_path)
                    }
                    Err(e) => FileProcessResult::Error(AppError::Io(e)),
                }
            }
            Err(e) => FileProcessResult::Error(AppError::Io(e)),
        }
    }
}

#[async_trait]
impl LinkOperations for Linker {
    async fn link_recursively(
        &self,
        source_path: &Path,
        target_path: &Path,
        force: bool,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let s = self.path_expander.parse_path(source_path).await?;
        let t = self.path_expander.parse_path(target_path).await?;

        let ans = self
            .confirmation(
                &format!(
                    "This will link files from {:?} to {:?}. Do you want to continue?",
                    s.display(),
                    t.display()
                ),
                "This will create symlinks in your home directory",
            )
            .await?;

        if !ans {
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        for entry in WalkDir::new(&s).into_iter().filter_map(Result::ok) {
            if !self.should_ignore(entry.path()) {
                results.push(
                    self.symlink_process(entry, s.as_path(), t.as_path(), force)
                        .await,
                );
            }
        }

        Ok(results)
    }

    async fn materialize_symlinks_recursively(
        &self,
        target_path: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let mut results = Vec::new();
        for entry in WalkDir::new(target_path).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !self.should_ignore(path) && path.is_symlink() {
                results.push(self.materialize_symlink(path).await);
            }
        }

        Ok(results)
    }

    async fn confirmation(&self, message: &str, help_message: &str) -> Result<bool, AppError> {
        Confirm::new(message)
            .with_help_message(help_message)
            .with_default(false)
            .prompt()
            .map_err(|e| AppError::InvalidInput(e.to_string()))
    }
}

pub async fn make_symlinks(
    linker: Arc<dyn LinkOperations>,
    source_path: &Path,
    force: bool,
    test: bool,
) -> Result<(), AppError> {
    let home = home_dir().ok_or(AppError::HomeDirectoryNotFound)?;
    let target_path = if test {
        home.join(Path::new(LINK_TEST_DIR))
    } else {
        home
    };

    let results = linker
        .link_recursively(source_path, target_path.as_path(), force)
        .await?;

    for result in results {
        match result {
            FileProcessResult::Linked(s, t) => {
                println!(
                    "Linked successfully: {:?} -> {:?}",
                    s.display(),
                    t.display()
                )
            }
            FileProcessResult::Created(p) => println!("Created directory: {:?}", p),
            FileProcessResult::Skipped(path) => println!("Skipped: {:?}", path),
            FileProcessResult::Error(e) => println!("Error: {:?}", e),
            FileProcessResult::Materialized(_, _) => unreachable!(),
        }
    }

    Ok(())
}
