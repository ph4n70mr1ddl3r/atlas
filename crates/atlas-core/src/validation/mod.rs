//! Validation Engine
//! 
//! Declarative validation rules for record data.

mod engine;
mod rules;

pub use engine::ValidationEngine;
pub use rules::*;

use atlas_shared::{AtlasError, AtlasResult};
use serde::{Deserialize, Serialize};

/// Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub organization_id: Option<uuid::Uuid>,
    pub user_id: Option<uuid::Uuid>,
    pub data: serde_json::Value,
}

/// A single validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub value: Option<serde_json::Value>,
}

/// Result of validation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self { valid: true, errors: vec![] }
    }
    
    pub fn add_error(&mut self, field: &str, code: &str, message: &str) {
        self.valid = false;
        self.errors.push(ValidationError {
            field: field.to_string(),
            code: code.to_string(),
            message: message.to_string(),
            value: None,
        });
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.valid {
            self.valid = false;
            self.errors.extend(other.errors);
        }
    }
    
    pub fn into_result(self) -> AtlasResult<()> {
        if self.valid {
            Ok(())
        } else {
            Err(AtlasError::ValidationFailed(
                serde_json::to_string(&self.errors).unwrap_or_default()
            ))
        }
    }
}
