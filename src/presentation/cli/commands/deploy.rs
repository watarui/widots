use crate::application::AppConfig;
use crate::error::AppError;

pub async fn execute(config: &AppConfig) -> Result<(), AppError> {
    config.get_deploy_service().build().await?;
    config.get_deploy_service().deploy().await?;
    println!("Deployed successfully");
    Ok(())
}
