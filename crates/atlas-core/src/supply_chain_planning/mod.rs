//! Supply Chain Planning / MRP Module
//!
//! Oracle Fusion Cloud: Supply Chain Management > Supply Chain Planning
//!
//! Provides Material Requirements Planning (MRP) capabilities:
//! - Planning scenarios (define planning run contexts)
//! - Planning parameters (item-level planning attributes)
//! - Supply/demand entries (netting inputs)
//! - Planned orders (MRP output)
//! - Planning exceptions (issues flagged during planning)
//!
//! Oracle Fusion equivalent: Supply Chain Management > Planning > MRP

mod repository;
pub mod engine;

pub use engine::SupplyChainPlanningEngine;
pub use repository::{PlanningRepository, PostgresPlanningRepository};
