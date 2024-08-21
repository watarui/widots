use crate::domain::shell::ShellExecutor;
use crate::error::AppError;
use async_trait::async_trait;
use std::process::Output;
use tokio::process::Command;

#[derive(Debug)]
pub struct SystemShellExecutor;

impl Default for SystemShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ShellExecutor for SystemShellExecutor {
    async fn execute<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<String, AppError> {
        let output = Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| AppError::ShellExecution(format!("Failed to execute command: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(AppError::ShellExecution(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ))
        }
    }

    async fn output<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<Output, AppError> {
        Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| AppError::ShellExecution(format!("Failed to execute command: {}", e)))
    }

    fn stderr(&self, output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{remove_file, File},
        io::Write,
        os::unix::process::ExitStatusExt,
        process::ExitStatus,
    };

    use super::*;
    use mockall::{mock, predicate::*};
    use proptest::prelude::*;
    use tokio::runtime::Runtime;

    mock! {
        pub SystemShellExecutor {}

        #[async_trait]
        impl ShellExecutor for SystemShellExecutor {
            async fn execute<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<String, AppError>;
            async fn output<'a>(&self, command: &'a str, args: &'a [&'a str]) -> Result<Output, AppError>;
            fn stderr(&self, output: &Output) -> String;
        }
    }

    #[test]
    fn test_output_success() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_output()
            .withf(|cmd: &str, args: &[&str]| cmd == "echo" && args == ["Hello"])
            .returning(|_, _| {
                Ok(Output {
                    status: ExitStatus::from_raw(0),
                    stdout: b"Hello\n".to_vec(),
                    stderr: vec![],
                })
            });

        let result = rt.block_on(mock.output("echo", &["Hello"]));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"Hello\n");
    }

    #[test]
    fn test_output_failure() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_output()
            .withf(|cmd: &str, args: &[&str]| cmd == "invalid_command" && args.is_empty())
            .returning(|_, _| Err(AppError::ShellExecution("Command not found".to_string())));

        let result = rt.block_on(mock.output("invalid_command", &[]));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ShellExecution(_)));
    }

    #[test]
    fn test_stderr() {
        let executor = SystemShellExecutor::new();
        let output = Output {
            status: ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: b"Error occurred".to_vec(),
        };

        let stderr = executor.stderr(&output);
        assert_eq!(stderr, "Error occurred");
    }

    #[test]
    fn test_execute_success_with_mock() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_execute()
            .withf(|cmd: &str, args: &[&str]| cmd == "echo" && args == ["Hello, World!"])
            .returning(|_, _| Ok("Hello, World!".to_string()));

        let result = rt.block_on(mock.execute("echo", &["Hello, World!"]));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_execute_failure_with_mock() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_execute()
            .withf(|cmd: &str, args: &[&str]| {
                cmd == "non_existent_command" && args == Vec::<&str>::new()
            })
            .returning(|_, _| Err(AppError::ShellExecution("Command not found".to_string())));

        let result = rt.block_on(mock.execute("non_existent_command", &[]));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ShellExecution(_)));
    }

    proptest! {
        #[test]
        fn doesnt_crash_on_any_command_and_args(command in "\\PC*", args in prop::collection::vec("\\PC*", 0..10)) {
            let rt = Runtime::new().unwrap();
            let executor = SystemShellExecutor::new();
            let args_slice: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

            let result = rt.block_on(executor.execute(&command, &args_slice));

            prop_assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_output_success_with_mock() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_output()
            .withf(|cmd: &str, args: &[&str]| cmd == "echo" && args == ["Hello, World!"])
            .returning(|_, _| {
                Ok(Output {
                    status: ExitStatus::from_raw(0), // 0 is usually the success exit code
                    stdout: b"Hello, World!".to_vec(),
                    stderr: vec![],
                })
            });

        let result = rt.block_on(mock.output("echo", &["Hello, World!"]));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, World!");
    }

    #[test]
    fn test_stderr_with_mock() {
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_stderr()
            .returning(|output| String::from_utf8_lossy(&output.stderr).to_string());

        let output = Output {
            status: ExitStatus::from_raw(1), // 1 is usually an error exit code
            stdout: vec![],
            stderr: b"Error message".to_vec(),
        };

        assert_eq!(mock.stderr(&output), "Error message");
    }

    #[test]
    fn test_execute_success_with_invalid_utf8() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_execute()
            .withf(|cmd: &str, args: &[&str]| cmd == "echo" && args == ["test"])
            .returning(|_, _| {
                Err(AppError::ShellExecution(
                    "Failed to parse command output: invalid utf-8 sequence".to_string(),
                ))
            });

        let result = rt.block_on(mock.execute("echo", &["test"]));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ShellExecution(_)));
    }

    #[test]
    fn test_execute_failure_with_stderr() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_execute()
            .withf(|cmd: &str, args: &[&str]| cmd == "invalid" && args.is_empty())
            .returning(|_, _| Err(AppError::ShellExecution("Command failed".to_string())));

        let result = rt.block_on(mock.execute("invalid", &[]));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::ShellExecution(msg) if msg == "Command failed"));
    }

    #[test]
    fn test_execute_success() {
        let rt = Runtime::new().unwrap();
        let mut mock = MockSystemShellExecutor::new();
        mock.expect_output()
            .withf(|cmd: &str, args: &[&str]| cmd == "echo" && args == ["Hello"])
            .returning(|_, _| {
                Ok(Output {
                    status: ExitStatus::from_raw(0),
                    stdout: b"Hello\n".to_vec(),
                    stderr: vec![],
                })
            });

        let result = rt.block_on(async {
            let executor = SystemShellExecutor::new();
            executor.execute("echo", &["Hello"]).await
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello\n");
    }

    #[test]
    fn test_execute_failure() {
        let rt = Runtime::new().unwrap();
        let executor = SystemShellExecutor::new();

        let result = rt.block_on(async { executor.execute("non_existent_command", &[]).await });

        assert!(result.is_err());
        match result {
            Err(AppError::ShellExecution(msg)) => {
                assert!(msg.starts_with("Failed to execute command:"));
                assert!(msg.contains("No such file or directory"));
            }
            _ => panic!("Expected AppError::ShellExecution, got {:?}", result),
        }
    }

    #[test]
    fn test_execute_invalid_utf8() {
        let rt = Runtime::new().unwrap();
        let executor = SystemShellExecutor::new();

        // 無効な UTF-8 シーケンスを含むファイルを作成
        let filename = "test_invalid_utf8.bin";
        let invalid_utf8 = [0xFF, 0xFE, 0xFD];
        {
            let mut file = File::create(filename).unwrap();
            file.write_all(&invalid_utf8).unwrap();
        }

        let result = rt.block_on(async { executor.execute("cat", &[filename]).await });

        // Cleanup the file after the test
        remove_file(filename).unwrap();

        assert!(
            result.is_ok(),
            "execute should succeed even with invalid UTF-8"
        );
        let output = result.unwrap();

        // Verification at the output level
        assert_eq!(
            output, "���",
            "Output should be three replacement characters"
        );
        assert_eq!(
            output,
            String::from_utf8_lossy(&invalid_utf8).to_string(),
            "Output should match String::from_utf8_lossy result"
        );

        // Verification at the byte level
        assert_eq!(
            output.len(),
            9,
            "Output should be 9 bytes long (3 bytes per replacement character)"
        );
        assert_eq!(
            output.as_bytes(),
            [0xEF, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD],
            "Output bytes should be the UTF-8 encoding of three replacement characters"
        );

        // Verification at the character level
        assert_eq!(
            output.chars().count(),
            3,
            "Output should contain 3 characters"
        );
        assert!(
            output.chars().all(|c| c == '\u{FFFD}'),
            "All characters should be the replacement character"
        );

        let char_vec: Vec<char> = output.chars().collect();
        assert_eq!(
            char_vec,
            vec!['\u{FFFD}', '\u{FFFD}', '\u{FFFD}'],
            "Output should be exactly three replacement characters"
        );
    }

    #[test]
    fn test_output_method() {
        let rt = Runtime::new().unwrap();
        let executor = SystemShellExecutor::new();

        let result = rt.block_on(executor.output("echo", &["Hello"]));

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello\n");
    }
}
