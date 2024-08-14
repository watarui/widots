#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io;
#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::io::{BufRead, BufReader};

use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use async_trait::async_trait;
use inquire::Confirm;

pub struct Prompt;

impl Prompt {
    pub fn new() -> Self {
        Prompt
    }
}

#[async_trait]
impl PromptOperations for Prompt {
    async fn confirm_action(&self, message: &str) -> Result<bool, AppError> {
        Confirm::new(message)
            .with_default(false)
            .prompt()
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}

#[cfg(test)]
pub struct DummyPrompt<R: BufRead> {
    reader: R,
}

#[cfg(test)]
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
    // テスト用の一時ファイルを作成し、ユーザー入力をシミュレート
    let temp_file = tempfile::NamedTempFile::new()?;
    writeln!(temp_file.as_file(), "y")?;

    // 標準入力を模倣
    let file = File::open(temp_file.path())?;
    let reader = BufReader::new(file);

    let mut prompt = DummyPrompt::new(reader);

    let result = prompt.confirm_action("Do you want to continue?")?;

    assert!(result);
    Ok(())
}
