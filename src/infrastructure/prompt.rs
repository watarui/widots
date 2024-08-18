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
    use std::fs::File;
    use std::io;
    use std::io::Write;
    use std::io::{BufRead, BufReader};

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
}
