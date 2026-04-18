//! Cost Allocation Module (Oracle Fusion GL > Allocations / Mass Allocations)
//!
//! Oracle Fusion Cloud ERP-inspired Cost Allocation Management.
//! Provides cost pool definitions, allocation bases (statistical & financial),
//! allocation rules with targets, rule execution with journal generation,
//! and allocation history tracking.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Allocations

mod repository;
pub mod engine;

pub use engine::CostAllocationEngine;
pub use repository::{CostAllocationRepository, PostgresCostAllocationRepository};
