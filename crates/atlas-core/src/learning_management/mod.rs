//! Learning Management Module
//!
//! Oracle Fusion HCM > Learning.
//! Provides:
//! - Learning items (courses, certifications, specializations, videos, assessments)
//! - Learning categories (hierarchical catalog organization)
//! - Learning enrollments (person enrollment and progress tracking)
//! - Learning paths (curricula / sequences of learning items)
//! - Learning path items (individual steps within a path)
//! - Learning assignments (mandatory training requirements)
//!
//! Oracle Fusion equivalent: HCM > Learning

mod repository;
pub mod engine;

pub use engine::LearningManagementEngine;
pub use repository::{LearningManagementRepository, PostgresLearningManagementRepository};
