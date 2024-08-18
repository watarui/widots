use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use async_trait::async_trait;
use inquire::Confirm;

#[derive(Debug)]
pub struct Prompt {
    force_yes: bool,
}

impl Default for Prompt {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Prompt {
    pub fn new(force_yes: bool) -> Self {
        Prompt { force_yes }
    }
}

#[async_trait]
impl PromptOperations for Prompt {
    async fn confirm_action(&self, message: &str) -> Result<bool, AppError> {
        if self.force_yes {
            println!("{}", message);
            Ok(true)
        } else {
            Confirm::new(message)
                .with_default(false)
                .prompt()
                .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::*;
    use predicate::eq;
    use proptest::prelude::*;
    use std::fs::File;
    use std::io;
    use std::io::Write;
    use std::io::{BufRead, BufReader};

    mock! {
        pub Prompt {}

        #[async_trait]
        impl PromptOperations for Prompt {
            async fn confirm_action(&self, message: &str) -> Result<bool, AppError>;
        }
    }

    #[test]
    fn test_toml_prompt_default() {
        let default_parser = Prompt { force_yes: false };
        let new_parser = Prompt::new(false);

        // Ensure that the default implementation works correctly
        assert_eq!(format!("{:?}", default_parser), format!("{:?}", new_parser));
    }

    pub struct DummyPrompt<R: BufRead> {
        reader: R,
    }

    impl<R: BufRead> DummyPrompt<R> {
        pub fn new(reader: R) -> Self {
            DummyPrompt { reader }
        }

        pub fn confirm_action(&mut self, message: &str) -> Result<bool, io::Error> {
            println!("{}", message);
            let mut input = String::new();
            self.reader.read_line(&mut input)?;
            Ok(input.trim() == "y")
        }
    }

    #[tokio::test]
    async fn test_confirm_action() -> Result<(), io::Error> {
        // Simulate user input by creating a temporary file for testing
        let temp_file = tempfile::NamedTempFile::new()?;
        writeln!(temp_file.as_file(), "y")?;

        // Emulate standard input
        let file = File::open(temp_file.path())?;
        let reader = BufReader::new(file);

        let mut prompt = DummyPrompt::new(reader);

        let result = prompt.confirm_action("Do you want to continue?")?;

        assert!(result);
        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_action_force_yes() {
        let prompt = Prompt::new(true);
        let result = prompt.confirm_action("Test action").await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_confirm_action_with_mock() {
        let mut mock = MockPrompt::new();
        mock.expect_confirm_action()
            .with(eq("Test action"))
            .times(1)
            .returning(|_| Ok(true));

        let result = mock.confirm_action("Test action").await.unwrap();
        assert!(result);
    }

    proptest! {
        #[test]
        fn test_confirm_action_with_various_messages(message in "[a-zA-Z0-9 ]{1,50}") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let prompt = Prompt::new(true);
                let result = prompt.confirm_action(&message).await;
                prop_assert!(result.is_ok(), "confirm_action failed");
                prop_assert!(result.unwrap(), "confirm_action returned false when force_yes is true");
                Ok(())
            }).unwrap()
        }
    }

    #[test]
    fn test_prompt_new_and_default() {
        let new_prompt = Prompt::new(false);
        let default_prompt = Prompt::default();

        assert_eq!(format!("{:?}", new_prompt), format!("{:?}", default_prompt));
    }
}
