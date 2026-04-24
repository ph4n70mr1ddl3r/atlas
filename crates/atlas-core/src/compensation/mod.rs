//! Compensation Management Module
//!
//! Oracle Fusion Cloud HCM Compensation Workbench implementation.
//! Manages compensation plans, cycles, budget pools, worksheets,
//! per-employee allocation lines, and compensation statements.
//!
//! Oracle Fusion equivalent: HCM > Compensation > Workforce Compensation

mod engine;
mod repository;

pub use engine::CompensationEngine;
pub use repository::{CompensationRepository, PostgresCompensationRepository};
