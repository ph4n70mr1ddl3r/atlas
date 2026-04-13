//! Atlas Core Engine
//! 
//! The declarative foundation of Atlas ERP. This module contains:
//! - Schema engine for dynamic entity definitions
//! - Workflow engine for state machine execution
//! - Validation engine for declarative rules
//! - Formula engine for computed fields
//! - Security engine for access control
//! - Audit engine for change tracking
//! - Configuration engine for hot-reload
//! - Event bus for inter-service communication

pub mod schema;
pub mod workflow;
pub mod validation;
pub mod formula;
pub mod security;
pub mod audit;
pub mod config;
pub mod eventbus;

pub use schema::*;
pub use workflow::*;
pub use validation::*;
pub use formula::*;
pub use security::*;
pub use audit::*;
pub use config::*;
pub use eventbus::*;

mod mock_repos;
pub use mock_repos::*;
