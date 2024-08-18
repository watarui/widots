use crate::domain::link::LinkOperations;
use crate::domain::os::OSOperations;
use crate::domain::path::PathOperations;
use crate::domain::prompt::PromptOperations;
use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use crate::models::config::Config;
use crate::models::link::FileProcessResult;
use crate::utils::toml::TomlOperations;
use async_trait::async_trait;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[async_trait]
pub trait LoadService: Send + Sync {
    async fn load(&self, config_path: &Path, target: &Path) -> Result<(), AppError>;
}

pub struct LoadServiceImpl {
    link_operations: Arc<dyn LinkOperations>,
    path_operations: Arc<dyn PathOperations>,
    toml_parser: Arc<dyn TomlOperations>,
    os_detector: Arc<dyn OSOperations>,
    shell_executor: Arc<dyn ShellExecutor>,
    prompter: Arc<dyn PromptOperations>,
}

impl LoadServiceImpl {
    pub fn new(
        link_operations: Arc<dyn LinkOperations>,
        path_operations: Arc<dyn PathOperations>,
        toml_parser: Arc<dyn TomlOperations>,
        os_detector: Arc<dyn OSOperations>,
        shell_executor: Arc<dyn ShellExecutor>,
        prompter: Arc<dyn PromptOperations>,
    ) -> Self {
        Self {
            link_operations,
            path_operations,
            toml_parser,
            os_detector,
            shell_executor,
            prompter,
        }
    }

    async fn evaluate_link_section(&self, config: &Config, target: &Path) -> Result<(), AppError> {
        if let Some(links) = &config.link {
            for link in links {
                self.link_dotfiles(&link.location, target).await?;
            }
        }
        Ok(())
    }

    async fn evaluate_provision_section(&self, config: &Config) -> Result<(), AppError> {
        if let Some(provisions) = &config.provision {
            for provision in provisions {
                if provision.mode == self.os_detector.get_os().await? {
                    println!("🏃 Run provisioning... for {}", provision.mode);
                    self.run_bash_script(&provision.script).await?;
                    println!("🚀 Provisioning done");
                }
            }
        }
        Ok(())
    }

    async fn run_bash_script(&self, script: &str) -> Result<(), AppError> {
        let mut temp_file = NamedTempFile::new().map_err(AppError::Io)?;
        temp_file
            .as_file_mut()
            .write_all(script.as_bytes())
            .map_err(AppError::Io)?;

        let command = format!("bash {}", temp_file.path().display());
        self.shell_executor.execute(&command).await?;
        Ok(())
    }

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

        self.link_operations
            .link_recursively(&source, &target)
            .await
    }
}

#[async_trait]
impl LoadService for LoadServiceImpl {
    async fn load(&self, config_path: &Path, target: &Path) -> Result<(), AppError> {
        let config = self.toml_parser.parse(config_path).await?;

        self.evaluate_link_section(&config, target).await?;
        self.evaluate_provision_section(&config).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::link::LinkOperations;
    use crate::domain::os::OSOperations;
    use crate::domain::path::PathOperations;
    use crate::domain::prompt::PromptOperations;
    use crate::domain::shell::ShellExecutor;
    use crate::error::AppError;
    use crate::models::config::Config;
    use crate::models::link::FileProcessResult;
    use crate::utils::toml::TomlOperations;
    use async_trait::async_trait;
    use mockall::mock;
    use serde::de::Error;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

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

    mock! {
        PathOperations {}
        #[async_trait]
        impl PathOperations for PathOperations {
            async fn expand_tilde(&self, path: &Path) -> Result<PathBuf, AppError>;
            async fn parse_path(&self, path: &Path) -> Result<PathBuf, AppError>;
            async fn get_home_dir(&self) -> Result<PathBuf, AppError>;
        }
    }

    mock! {
        TomlOperations {}
        #[async_trait]
        impl TomlOperations for TomlOperations {
            async fn parse(&self, path: &Path) -> Result<Config, AppError>;
        }
    }

    mock! {
        OSOperations {}
        #[async_trait]
        impl OSOperations for OSOperations {
            async fn get_os(&self) -> Result<String, AppError>;
        }
    }

    mock! {
        ShellExecutor {}
        #[async_trait]
        impl ShellExecutor for ShellExecutor {
            async fn execute(&self, command: &str) -> Result<String, AppError>;
            async fn output(&self, command: &str) -> Result<std::process::Output, AppError>;
            fn stderr(&self, output: &std::process::Output) -> String;
        }
    }

    mock! {
        PromptOperations {}
        #[async_trait]
        impl PromptOperations for PromptOperations {
            async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
        }
    }

    #[tokio::test]
    async fn test_load() {
        let mut mock_link_ops = MockLinkOperations::new();
        let mut mock_path_ops = MockPathOperations::new();
        let mut mock_toml_ops = MockTomlOperations::new();
        let mut mock_os_ops = MockOSOperations::new();
        let mock_shell = MockShellExecutor::new();
        let mut mock_prompt_ops = MockPromptOperations::new();

        mock_path_ops
            .expect_parse_path()
            .returning(|path| Ok(path.to_path_buf()));

        mock_toml_ops
            .expect_parse()
            .returning(|_| Ok(Config::default()));

        mock_os_ops
            .expect_get_os()
            .returning(|| Ok("macos".to_string()));

        mock_prompt_ops
            .expect_confirm_action()
            .returning(|_| Ok(true));

        mock_link_ops
            .expect_link_recursively()
            .returning(|_, _| Ok(vec![]));

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service
            .load(Path::new("/config.toml"), Path::new("/target"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_with_provision() {
        let mock_link_ops = MockLinkOperations::new();
        let mut mock_path_ops = MockPathOperations::new();
        let mut mock_toml_ops = MockTomlOperations::new();
        let mut mock_os_ops = MockOSOperations::new();
        let mut mock_shell = MockShellExecutor::new();
        let mut mock_prompt_ops = MockPromptOperations::new();

        mock_path_ops
            .expect_parse_path()
            .returning(|path| Ok(path.to_path_buf()));

        mock_toml_ops.expect_parse().returning(|_| {
            Ok(Config {
                provision: Some(vec![crate::models::config::Provision {
                    mode: "macos".to_string(),
                    script: "echo 'Hello, World!'".to_string(),
                }]),
                ..Default::default()
            })
        });

        mock_os_ops
            .expect_get_os()
            .returning(|| Ok("macos".to_string()));

        mock_prompt_ops
            .expect_confirm_action()
            .returning(|_| Ok(true));

        mock_shell
            .expect_execute()
            .returning(|_| Ok("Provision executed successfully".to_string()));

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service
            .load(Path::new("/config.toml"), Path::new("/target"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_with_invalid_config() {
        let mock_link_ops = MockLinkOperations::new();
        let mock_path_ops = MockPathOperations::new();
        let mut mock_toml_ops = MockTomlOperations::new();
        let mock_os_ops = MockOSOperations::new();
        let mock_shell = MockShellExecutor::new();
        let mock_prompt_ops = MockPromptOperations::new();

        mock_toml_ops
            .expect_parse()
            .returning(|_| Err(AppError::TomlParse(toml::de::Error::custom("Invalid TOML"))));

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service
            .load(Path::new("/config.toml"), Path::new("/target"))
            .await;

        println!("{:?}", result);
        assert!(matches!(result, Err(AppError::TomlParse(_))));
    }

    #[tokio::test]
    async fn test_load_with_provision_different_os() {
        let mock_link_ops = MockLinkOperations::new();
        let mut mock_path_ops = MockPathOperations::new();
        let mut mock_toml_ops = MockTomlOperations::new();
        let mut mock_os_ops = MockOSOperations::new();
        let mock_shell = MockShellExecutor::new();
        let mock_prompt_ops = MockPromptOperations::new();

        mock_path_ops
            .expect_parse_path()
            .returning(|path| Ok(path.to_path_buf()));

        mock_toml_ops.expect_parse().returning(|_| {
            Ok(Config {
                provision: Some(vec![crate::models::config::Provision {
                    mode: "linux".to_string(),
                    script: "echo 'Hello, World!'".to_string(),
                }]),
                ..Default::default()
            })
        });

        mock_os_ops
            .expect_get_os()
            .returning(|| Ok("macos".to_string()));

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service
            .load(Path::new("/config.toml"), Path::new("/target"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_with_links() {
        let mut mock_link_ops = MockLinkOperations::new();
        let mut mock_path_ops = MockPathOperations::new();
        let mut mock_toml_ops = MockTomlOperations::new();
        let mock_os_ops = MockOSOperations::new();
        let mock_shell = MockShellExecutor::new();
        let mut mock_prompt_ops = MockPromptOperations::new();

        mock_path_ops
            .expect_parse_path()
            .returning(|path| Ok(path.to_path_buf()));

        mock_toml_ops.expect_parse().returning(|_| {
            Ok(Config {
                link: Some(vec![crate::models::config::Link {
                    location: PathBuf::from("/source"),
                }]),
                ..Default::default()
            })
        });

        mock_prompt_ops
            .expect_confirm_action()
            .returning(|_| Ok(true));

        mock_link_ops.expect_link_recursively().returning(|_, _| {
            Ok(vec![FileProcessResult::Linked(
                PathBuf::from("/source/file"),
                PathBuf::from("/target/file"),
            )])
        });

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service
            .load(Path::new("/config.toml"), Path::new("/target"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_link_dotfiles_user_declines() {
        let mock_link_ops = MockLinkOperations::new();
        let mut mock_path_ops = MockPathOperations::new();
        let mock_toml_ops = MockTomlOperations::new();
        let mock_os_ops = MockOSOperations::new();
        let mock_shell = MockShellExecutor::new();
        let mut mock_prompt_ops = MockPromptOperations::new();

        mock_path_ops
            .expect_parse_path()
            .returning(|path| Ok(path.to_path_buf()));

        mock_prompt_ops
            .expect_confirm_action()
            .returning(|_| Ok(false));

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service
            .link_dotfiles(Path::new("/source"), Path::new("/target"))
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty()); // Check if the vector is empty instead of using ==
    }

    #[tokio::test]
    async fn test_run_bash_script_io_error() {
        let mock_link_ops = MockLinkOperations::new();
        let mock_path_ops = MockPathOperations::new();
        let mock_toml_ops = MockTomlOperations::new();
        let mock_os_ops = MockOSOperations::new();
        let mut mock_shell = MockShellExecutor::new();
        let mock_prompt_ops = MockPromptOperations::new();

        mock_shell.expect_execute().returning(|_| {
            Err(AppError::ShellExecution(
                "Failed to execute script".to_string(),
            ))
        });

        let load_service = LoadServiceImpl::new(
            Arc::new(mock_link_ops),
            Arc::new(mock_path_ops),
            Arc::new(mock_toml_ops),
            Arc::new(mock_os_ops),
            Arc::new(mock_shell),
            Arc::new(mock_prompt_ops),
        );

        let result = load_service.run_bash_script("echo 'test'").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::ShellExecution(_))));
    }
}
