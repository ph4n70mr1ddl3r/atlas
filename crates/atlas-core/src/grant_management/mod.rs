//! Grant Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Grant Management.
//! Manages grant sponsors, award lifecycle, budgets, expenditures,
//! sponsor billing, compliance reporting, and indirect cost calculations.
//!
//! Oracle Fusion equivalent: Financials > Grants Management > Awards

pub mod engine;
mod repository;

pub use engine::GrantManagementEngine;
pub use repository::{GrantManagementRepository, PostgresGrantManagementRepository};
