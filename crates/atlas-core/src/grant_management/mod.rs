//! Grant Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Grant Management.
//! Manages grant sponsors, award lifecycle, budgets, expenditures,
//! sponsor billing, compliance reporting, and indirect cost calculations.
//!
//! Oracle Fusion equivalent: Financials > Grants Management > Awards

mod repository;
pub mod engine;

pub use engine::GrantManagementEngine;
pub use repository::{GrantManagementRepository, PostgresGrantManagementRepository};
