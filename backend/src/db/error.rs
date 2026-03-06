use serde_json::{Map, Value};

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum RepositoryError {
    #[error("Repository Error: Conflict")]
    ResourceConflict,
    #[error("Repository Error: Not Found")]
    ResourceNotFound,
    #[error("Repository Error: Validation")]
    ResourceValidation { detail: Option<Map<String, Value>> },
    #[error("Repository Error: Driver<{driver}> {error}")]
    DriverError { driver: &'static str, error: Box<dyn std::error::Error> },
}

impl RepositoryError {

    pub fn is_driver_error(&self) -> bool {
        match self {
            Self::DriverError { .. } => true,
            _ => false
        }
    }

}
