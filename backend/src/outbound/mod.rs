pub mod interfaces;
pub mod pg;

/// A driver that provides access to external resources through a standardized interface.
///
/// Drivers act as bridges between the system and external services, databases, or APIs.
/// They encapsulate the logic for connecting to, authenticating with, and operating on
/// external resources while providing a consistent interface for the rest of the application.
///
/// **Requirements**
///
/// Implementations of this trait must:
/// * Be thread-safe (`Send + Sync`)
/// * Have a static lifetime (`'static`)
/// * Provide a unique name that identifies the driver
/// * Implement all *interfaces*
///
/// Note: we borrow the 'Driver' terminology because it sounds cool
/// 
pub trait Driver: Send + Sync + 'static
{

    /// Returns the unique identifier for this driver.
    fn name(&self) -> &'static str;

}

pub async fn from_env() -> Result<std::sync::Arc<dyn Driver>, Box<dyn std::error::Error>> {
    let driver = pg::PgDriver::from_env().await?;
    Ok(std::sync::Arc::new(driver))
}
