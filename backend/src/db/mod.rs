use std::sync::Arc;
pub mod error;
pub mod interfaces;
pub mod pg;

pub trait Repository
    : Send
    + Sync
    + 'static
    + interfaces::example::IExample
{}

pub async fn from_env() -> Result<Arc<dyn Repository>, Box<dyn std::error::Error>> {
    let driver = pg::PgDriver::from_env().await?;
    Ok(Arc::new(driver))
}
