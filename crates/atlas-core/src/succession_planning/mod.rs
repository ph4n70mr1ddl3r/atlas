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

mod repository;
pub mod engine;

pub use engine::SuccessionPlanningEngine;
pub use repository::{SuccessionPlanningRepository, PostgresSuccessionPlanningRepository};
