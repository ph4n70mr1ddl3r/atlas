//! Atlas Error Types

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for Atlas operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AtlasError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Field not found: {0}.{1}")]
    FieldNotFound(String, String),

    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),

    #[error("Invalid state transition: {0} -> {1}")]
    InvalidStateTransition(String, String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Event bus error: {0}")]
    EventBusError(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for AtlasError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::EntityNotFound("Record not found".to_string()),
            _ => Self::DatabaseError(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AtlasError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Unauthorized(err.to_string())
    }
}

/// Result type alias for Atlas operations
pub type AtlasResult<T> = Result<T, AtlasError>;

/// HTTP status code mapping
impl AtlasError {
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::EntityNotFound(_) => 404,
            Self::FieldNotFound(_, _) => 404,
            Self::ValidationFailed(_) => 400,
            Self::WorkflowError(_) => 400,
            Self::InvalidStateTransition(_, _) => 400,
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::ConfigError(_) => 500,
            Self::DatabaseError(_) => 500,
            Self::EventBusError(_) => 500,
            Self::SchemaError(_) => 500,
            Self::NotImplemented(_) => 501,
            Self::Conflict(_) => 409,
            Self::InvalidFieldType(_) => 400,
            Self::Internal(_) => 500,
        }
    }
}
