//! Goal Management Module
//!
//! Oracle Fusion HCM > Goal Management.
//! Provides:
//! - Goal library categories and templates (predefined goal definitions)
//! - Goal plans (performance/development periods)
//! - Goals with cascading hierarchy (org → team → individual)
//! - Progress tracking with target/actual metrics
//! - Goal alignments (explicit links between goals)
//! - Goal notes (comments, feedback, check-ins)
//!
//! Oracle Fusion equivalent: HCM > Goal Management

pub mod engine;
mod repository;

pub use engine::GoalManagementEngine;
pub use repository::{GoalManagementRepository, PostgresGoalManagementRepository};
