//! Validation Rules Module
//!
//! Built-in validation rules.

use atlas_shared::ValidationRule;
use serde::{Deserialize, Serialize};

/// Built-in validation rule types
pub fn builtin_rules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("required", "Field cannot be empty"),
        ("email", "Must be a valid email address"),
        ("url", "Must be a valid URL"),
        ("phone", "Must be a valid phone number"),
        ("date", "Must be a valid date"),
        ("number", "Must be a number"),
        ("integer", "Must be an integer"),
        ("positive", "Must be a positive number"),
        ("negative", "Must be a negative number"),
    ]
}
