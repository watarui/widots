use crate::constants::{LINK_IGNORED_ANCESTORS, LINK_IGNORED_FILES, LINK_IGNORED_PREFIXES};
use crate::domain::link::LinkOperations;
use crate::error::AppError;
use crate::models::link::FileProcessResult;
use async_trait::async_trait;
use regex::Regex;
use std::path::{Component, Path};
use tokio::fs;

pub struct LinkerImpl;

impl Default for LinkerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkerImpl {
    pub fn new() -> Self {
        Self
    }

    fn validate_filename(&self, filename: &str) -> Result<(), String> {
        if filename.is_empty() {
            return Err("Filename cannot be empty".to_string());
        }

        if filename == "." || filename == ".." {
            return Err("Filename cannot be '.' or '..'".to_string());
        }

        let valid_chars = Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
        if !valid_chars.is_match(filename) {
            return Err("Filename contains invalid characters".to_string());
        }

        Ok(())
    }

    fn is_ignored_file(&self, file_name: &str) -> bool {
        LINK_IGNORED_FILES.contains(file_name)
    }

    fn has_ignored_prefix(&self, file_name: &str) -> bool {
        LINK_IGNORED_PREFIXES
            .iter()
            .any(|prefix| file_name.starts_with(prefix))
    }

    fn has_ignored_ancestor(&self, path: &Path) -> bool {
        path.ancestors().any(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|name| LINK_IGNORED_ANCESTORS.contains(name))
                .unwrap_or(false)
        })
    }

    fn is_git_ignore_or_config(&self, path: &Path) -> bool {
        let components: Vec<_> = path.components().collect();
        let len = components.len();

        if len >= 2 {
            match (&components[len - 2], &components[len - 1]) {
                (Component::Normal(dir), Component::Normal(file)) => {
                    let dir = dir.to_str().unwrap_or("");
                    let file = file.to_str().unwrap_or("");
                    dir == "git" && (file == "ignore" || file == "config")
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

#[async_trait]
impl LinkOperations for LinkerImpl {
    async fn link_recursively(
        &self,
        source: &Path,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let mut results = Vec::new();

        if !target.exists() {
            tokio::fs::create_dir_all(target).await?;
            results.push(FileProcessResult::Created(target.to_path_buf()));
        }

        let mut entries = tokio::fs::read_dir(source).await?;
        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().ok_or_else(|| {
                AppError::InvalidFilename(file_name.to_string_lossy().to_string())
            })?;

            if let Err(e) = self.validate_filename(file_name_str) {
                println!("Skipped invalid file name: {} due to {}", file_name_str, e);
                results.push(FileProcessResult::Skipped(src_path));
                continue;
            }

            if self.should_ignore(&src_path) {
                println!("Ignored: {:?}", src_path);
                results.push(FileProcessResult::Skipped(src_path));
                continue;
            }

            let dst_path = target.join(&file_name);

            if src_path.is_dir() {
                tokio::fs::create_dir_all(&dst_path).await?;
                let mut sub_results = self.link_recursively(&src_path, &dst_path).await?;
                results.append(&mut sub_results);
            } else {
                if dst_path.exists() {
                    tokio::fs::remove_file(&dst_path).await?;
                }
                tokio::fs::symlink(&src_path, &dst_path).await?;
                results.push(FileProcessResult::Linked(src_path, dst_path));
            }
        }

        Ok(results)
    }

    async fn materialize_symlinks_recursively(
        &self,
        target: &Path,
    ) -> Result<Vec<FileProcessResult>, AppError> {
        let mut results = Vec::new();
        let mut dirs = vec![target.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(&dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    dirs.push(path);
                } else if path.is_symlink() {
                    let target = fs::read_link(&path).await?;
                    fs::remove_file(&path).await?;
                    fs::copy(&target, &path).await?;
                    results.push(FileProcessResult::Materialized(path, target));
                }
            }
        }

        Ok(results)
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        self.is_ignored_file(file_name)
            || self.has_ignored_prefix(file_name)
            || self.has_ignored_ancestor(path)
            || self.is_git_ignore_or_config(path)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use prop::string::string_regex;
    use proptest::prelude::*;
    use proptest::strategy::Strategy;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_should_ignore() {
        let linker = LinkerImpl::new();

        // Test ignored files
        assert!(linker.should_ignore(Path::new(".DS_Store")));
        assert!(linker.should_ignore(Path::new(".gitignore")));

        // Test ignored prefixes
        assert!(linker.should_ignore(Path::new(".hidden_file")));
        assert!(linker.should_ignore(Path::new("_ignored_file")));

        // Test ignored ancestors
        assert!(linker.should_ignore(Path::new("some/path/.git/config")));
        assert!(linker.should_ignore(Path::new("project/node_modules/package/file.js")));

        // Test git ignore or config
        assert!(linker.should_ignore(Path::new("some/path/git/ignore")));
        assert!(linker.should_ignore(Path::new("another/path/git/config")));
        assert!(linker.should_ignore(Path::new(".git/ignore")));
        assert!(linker.should_ignore(Path::new(".git/config")));

        // Test non-ignored files
        assert!(!linker.should_ignore(Path::new("README.md")));
        assert!(!linker.should_ignore(Path::new("some/path/file.rs")));

        // Test non-ignored git/ignore or git/config like files
        assert!(!linker.should_ignore(Path::new("some/path/git/other_file")));
        assert!(!linker.should_ignore(Path::new("foogit/ignore")));
        assert!(!linker.should_ignore(Path::new("git/ignorebar")));
        assert!(!linker.should_ignore(Path::new("some/git/path/ignore")));
    }

    #[test]
    fn test_validate_filename() {
        let linker = LinkerImpl::new();

        assert!(linker.validate_filename("valid_file.txt").is_ok());
        assert!(linker.validate_filename("another-valid-file.rs").is_ok());

        assert!(linker.validate_filename("").is_err());
        assert!(linker.validate_filename(".").is_err());
        assert!(linker.validate_filename("..").is_err());
        assert!(linker.validate_filename("invalid*file.txt").is_err());
        assert!(linker.validate_filename("invalid/file.txt").is_err());
    }

    #[test]
    fn test_validate_filename_special_entries() {
        let linker = LinkerImpl::new();
        assert!(linker.validate_filename(".").is_err());
        assert!(linker.validate_filename("..").is_err());
    }

    #[tokio::test]
    async fn test_link_recursively() -> Result<(), AppError> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");

        fs::create_dir(&source_dir).await?;
        fs::create_dir(&target_dir).await?;

        fs::write(source_dir.join("file1.txt"), "content1").await?;
        fs::write(source_dir.join("file2.txt"), "content2").await?;
        let dir1_path = source_dir.join("dir1");
        if !dir1_path.exists() {
            fs::create_dir_all(&dir1_path).await?;
        }
        fs::write(dir1_path.join("file3.txt"), "content3").await?;

        let linker = LinkerImpl::new();

        let results = linker.link_recursively(&source_dir, &target_dir).await?;

        assert_eq!(results.len(), 3);
        for result in results {
            match result {
                FileProcessResult::Linked(_, _) => {}
                FileProcessResult::Created(_) => {}
                _ => panic!("Expected Linked result"),
            }
        }

        assert!(fs::symlink_metadata(target_dir.join("file1.txt"))
            .await?
            .is_symlink());
        assert!(fs::symlink_metadata(target_dir.join("file2.txt"))
            .await?
            .is_symlink());
        assert!(fs::symlink_metadata(target_dir.join("dir1/file3.txt"))
            .await?
            .is_symlink());

        Ok(())
    }

    fn valid_filename() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9][a-zA-Z0-9_\-\.]{0,9}".prop_map(|s| {
            if s == "." || s == ".." {
                s + "x"
            } else {
                s
            }
        })
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
        fn test_should_ignore_prop(path in any::<PathBuf>()) {
            let linker = LinkerImpl::new();
            let ignored = linker.should_ignore(&path);

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let expected_ignore = LINK_IGNORED_FILES.contains(file_name)
                || LINK_IGNORED_PREFIXES.iter().any(|prefix| file_name.starts_with(prefix))
                || path.ancestors().any(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| LINK_IGNORED_ANCESTORS.contains(name))
                        .unwrap_or(false)
                })
                || linker.is_git_ignore_or_config(&path);

            prop_assert_eq!(ignored, expected_ignore);
        }

        #[test]
        fn test_validate_filename_prop(filename in r"[a-zA-Z0-9][a-zA-Z0-9._-]{0,254}") {
            let linker = LinkerImpl::new();
            if filename != "." && filename != ".." {
                prop_assert!(linker.validate_filename(&filename).is_ok());
            } else {
                prop_assert!(linker.validate_filename(&filename).is_err());
            }
        }

        #[test]
        fn test_invalid_filename_prop(filename in r"[^a-zA-Z0-9._-]{1,255}") {
            let linker = LinkerImpl::new();
            prop_assert!(linker.validate_filename(&filename).is_err());
        }

        #[test]
        fn test_link_recursively_prop(
            file_structure in prop::collection::hash_set(file_name_strategy(), 1..10).prop_flat_map(|names| {
                let names_vec: Vec<_> = names.into_iter().collect();
                let (dirs, files): (Vec<_>, Vec<_>) = names_vec.into_iter().enumerate()
                    .partition(|(i, _)| i % 2 == 0);
                let dirs = dirs.into_iter().map(|(_, name)| format!("dir_{}", name)).collect::<HashSet<_>>();
                let files = files.into_iter().map(|(_, name)| format!("file_{}", name)).collect::<HashSet<_>>();
                (Just(dirs), Just(files))
            }),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let source_dir = temp_dir.path().join("source");
                let target_dir = temp_dir.path().join("target");

                fs::create_dir_all(&source_dir).await.unwrap();
                fs::create_dir_all(&target_dir).await.unwrap();

                let (dirs, files) = file_structure;

                // Create directories first
                for dir in &dirs {
                    let full_path = source_dir.join(dir);
                    if let Err(e) = fs::create_dir_all(&full_path).await {
                        println!("Error creating directory {:?}: {:?}", full_path, e);
                    }
                }

                // Then create files
                for file in &files {
                    let full_path = source_dir.join(file);
                    if let Err(e) = fs::write(&full_path, "content").await {
                        println!("Error writing file {:?}: {:?}", full_path, e);
                    }
                }

                let linker = LinkerImpl::new();
                let results = match linker.link_recursively(&source_dir, &target_dir).await {
                    Ok(r) => r,
                    Err(e) => {
                        println!("Error in link_recursively: {:?}", e);
                        return Ok(());
                    }
                };

                // Check that all non-ignored files are linked
                // Compare file names case-insensitively
                let mut linked_files = HashSet::new();
                for result in &results {
                    if let FileProcessResult::Linked(source, _) = result {
                        linked_files.insert(source.strip_prefix(&source_dir).unwrap().to_string_lossy().to_lowercase());
                    }
                }

                for file in &files {
                    let path = Path::new(file);
                    if !linker.should_ignore(path) {
                        prop_assert!(linked_files.contains(&path.to_string_lossy().to_lowercase()), "Non-ignored file {:?} was not linked", path);
                    } else {
                        prop_assert!(!linked_files.contains(&path.to_string_lossy().to_lowercase()), "Ignored file {:?} was linked", path);
                    }
                }

                Ok(())
            }).unwrap()
        }
    }
}
