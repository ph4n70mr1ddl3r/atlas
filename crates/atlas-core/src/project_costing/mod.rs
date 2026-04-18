//! Project Costing Module (Oracle Fusion Cloud ERP)
//!
//! Oracle Fusion Cloud ERP-inspired Project Costing.
//! Provides cost transaction tracking against projects/tasks, burden schedule
//! management with overhead allocation, cost adjustments, and GL cost
//! distribution.
//!
//! Oracle Fusion equivalent: Project Management > Project Costing

mod repository;
pub mod engine;

pub use engine::ProjectCostingEngine;
pub use repository::{ProjectCostingRepository, PostgresProjectCostingRepository};
