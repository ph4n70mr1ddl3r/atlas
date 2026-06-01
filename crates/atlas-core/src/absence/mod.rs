//! Absence Management Module
//!
//! Oracle Fusion Cloud HCM-inspired Absence Management.
//! Provides absence types, accrual-based absence plans, employee absence
//! entries, balance tracking, and approval workflows.
//!
//! Oracle Fusion equivalent: HCM > Absence Management > Absence Types, Plans, Entries

pub mod engine;
mod repository;

pub use engine::AbsenceEngine;
pub use repository::{AbsenceRepository, PostgresAbsenceRepository};
