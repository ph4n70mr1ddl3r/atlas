//! Quality Management Module
//!
//! Oracle Fusion Cloud Quality Management implementation.
//! Provides inspection plan templates, quality inspections, non-conformance
//! reports (NCRs), corrective & preventive actions (CAPA), quality holds,
//! and statistical process control dashboards.
//!
//! Oracle Fusion equivalent: Quality Management > Inspections > Non-Conformance > CAPA

mod engine;
mod repository;

pub use engine::QualityManagementEngine;
pub use repository::{QualityManagementRepository, PostgresQualityManagementRepository};
