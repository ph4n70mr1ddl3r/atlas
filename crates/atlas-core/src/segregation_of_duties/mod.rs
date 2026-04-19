//! Segregation of Duties Module
//!
//! Oracle Fusion Cloud ERP-inspired Segregation of Duties (SoD).
//! Provides compliance enforcement by defining incompatible role/duty pairs,
//! detecting conflicts in user role assignments, managing mitigating controls
//! for exceptions, and reporting on violations.
//!
//! Key capabilities:
//! - Define SoD rules with incompatible duty sets and risk levels
//! - Track role assignments for SoD analysis
//! - Detect violations when users hold conflicting duties
//! - Check proposed role assignments for conflicts before assignment
//! - Manage mitigating controls as compensating measures
//! - Preventive mode: block conflicting assignments
//! - Detective mode: report conflicts without blocking
//!
//! Oracle Fusion equivalent: Advanced Access Control > Segregation of Duties

mod repository;
pub mod engine;

pub use engine::SegregationOfDutiesEngine;
pub use repository::{SegregationOfDutiesRepository, PostgresSegregationOfDutiesRepository};
