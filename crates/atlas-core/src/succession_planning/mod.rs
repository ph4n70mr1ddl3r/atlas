//! Succession Planning Module
//!
//! Oracle Fusion HCM > Succession Management.
//! Provides:
//! - Succession plans (for key positions with backup candidates)
//! - Talent pools (groups of high-potential employees)
//! - Talent reviews (formal assessment meetings with nine-box grid)
//! - Career paths (defined progression paths between jobs/roles)
//!
//! Oracle Fusion equivalent: HCM > Succession Management

pub mod engine;
mod repository;

pub use engine::SuccessionPlanningEngine;
pub use repository::{PostgresSuccessionPlanningRepository, SuccessionPlanningRepository};
