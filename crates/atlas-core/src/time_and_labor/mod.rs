//! Time and Labor Management Module
//!
//! Oracle Fusion Cloud HCM-inspired Time and Labor.
//! Provides work schedules, overtime rules, time cards with entries,
//! labor distribution, and approval workflows.
//!
//! Oracle Fusion equivalent: HCM > Time and Labor > Time Cards, Schedules, Entries

pub mod engine;
mod repository;

pub use engine::TimeAndLaborEngine;
pub use repository::{PostgresTimeAndLaborRepository, TimeAndLaborRepository};
