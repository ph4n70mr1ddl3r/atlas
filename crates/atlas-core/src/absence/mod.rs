//! Absence Management Module
//!
//! Oracle Fusion Cloud HCM-inspired Absence Management.
//! Provides absence types, accrual-based absence plans, employee absence
//! entries, balance tracking, and approval workflows.
//!
//! Oracle Fusion equivalent: HCM > Absence Management > Absence Types, Plans, Entries

mod repository;
pub mod engine;

pub use engine::AbsenceEngine;
pub use repository::{AbsenceRepository, PostgresAbsenceRepository};
