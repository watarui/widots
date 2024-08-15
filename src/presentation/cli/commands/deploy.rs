use crate::application::service_provider::ServiceProvider;
use crate::error::AppError;

pub async fn execute(services: &dyn ServiceProvider) -> Result<(), AppError> {
    services.deploy_service().execute().await?;
    println!("Deployed successfully");
    Ok(())
}
