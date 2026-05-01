//! Project Resource Management Module
//!
//! Oracle Fusion Cloud: Project Management > Resource Management
//! Provides:
//! - Resource profile management (employees/contractors, skills, rates)
//! - Resource request lifecycle (draft -> submitted -> fulfilled -> cancelled)
//! - Resource assignment management with planned/actual hours
//! - Utilization tracking with approval workflow
//! - Resource analytics dashboard
//!
//! Oracle Fusion equivalent: Project Management > Resource Management

mod repository;
pub mod engine;

pub use engine::ProjectResourceManagementEngine;
pub use repository::{ProjectResourceManagementRepository, PostgresProjectResourceManagementRepository};
