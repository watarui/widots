use crate::domain::prompt::PromptOperations;
use crate::error::AppError;
use crate::infrastructure::prompt::Prompt;
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_confirm_action() -> Result<(), AppError> {
    let prompt = Prompt::new();

    // テスト用の一時ファイルを作成し、ユーザー入力をシミュレート
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "y")?;
    temp_file.flush()?;

    // 標準入力を一時ファイルにリダイレクト
    let stdin = std::io::stdin();
    let guard = stdin.lock();
    std::io::set_stdin(temp_file);

    let result = prompt.confirm_action("Do you want to continue?").await?;

    // 標準入力を元に戻す
    drop(guard);

    assert!(result);
    Ok(())
}
