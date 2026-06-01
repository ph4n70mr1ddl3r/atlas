//! Performance Management Module
//!
//! Oracle Fusion Cloud HCM-inspired Performance Management.
//! Provides rating models, review cycles, performance documents, goals,
//! competency assessments, 360-degree feedback, and performance dashboards.
//!
//! Oracle Fusion equivalent: My Client Groups > Performance

pub mod engine;
mod repository;

pub use engine::PerformanceEngine;
pub use repository::{PerformanceRepository, PostgresPerformanceRepository};
