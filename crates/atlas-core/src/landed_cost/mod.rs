//! Landed Cost Management Module
//!
//! Oracle Fusion Cloud SCM-inspired Landed Cost Management.
//! Provides cost templates, cost components, charge capture,
//! cost allocation to receipt lines, landed cost simulation,
//! and variance analysis between estimated and actual costs.
//!
//! Oracle Fusion equivalent: SCM > Landed Cost Management

mod repository;
pub mod engine;

pub use engine::LandedCostEngine;
pub use repository::{LandedCostRepository, PostgresLandedCostRepository};
