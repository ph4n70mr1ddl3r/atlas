//! Atlas Error Types

use thiserror::Error;
use serde::{Deserialize, Serialize};

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
            sqlx::Error::RowNotFound => AtlasError::EntityNotFound("Record not found".to_string()),
            _ => AtlasError::DatabaseError(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AtlasError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AtlasError::Unauthorized(err.to_string())
    }
}

/// Result type alias for Atlas operations
pub type AtlasResult<T> = Result<T, AtlasError>;

/// HTTP status code mapping
impl AtlasError {
    pub fn status_code(&self) -> u16 {
        match self {
            AtlasError::EntityNotFound(_) => 404,
            AtlasError::FieldNotFound(_, _) => 404,
            AtlasError::ValidationFailed(_) => 400,
            AtlasError::WorkflowError(_) => 400,
            AtlasError::InvalidStateTransition(_, _) => 400,
            AtlasError::Unauthorized(_) => 401,
            AtlasError::Forbidden(_) => 403,
            AtlasError::ConfigError(_) => 500,
            AtlasError::DatabaseError(_) => 500,
            AtlasError::EventBusError(_) => 500,
            AtlasError::SchemaError(_) => 500,
            AtlasError::NotImplemented(_) => 501,
            AtlasError::Conflict(_) => 409,
            AtlasError::InvalidFieldType(_) => 400,
            AtlasError::Internal(_) => 500,
        }
    }
}
