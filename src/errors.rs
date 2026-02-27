use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Invalid Input, cannot be processed: {field} - {message}")]
    UnProcessableEntity { field: String, message: String },
    #[error("Environement Variable is missing: {0}")]
    MissingEnvironmentVarible(String),
    #[error("Failed to Parse: {0}")]
    ParsingError(String),
    #[error("You dont have permission to perform this action: {0}")]
    UnAuthorized(String),
}
