//! Territory Management Module
//!
//! Oracle Fusion CX Sales > Territory Management.
//! Manages sales territories, hierarchy, member assignments,
//! territory-based routing rules, and territory quotas.
//!
//! Oracle Fusion equivalent: CX Sales > Territory Management

mod engine;
mod repository;

pub use engine::TerritoryManagementEngine;
pub use repository::{TerritoryManagementRepository, PostgresTerritoryManagementRepository};
