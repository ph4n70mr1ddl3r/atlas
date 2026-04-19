//! General Ledger Allocations Module
//!
//! Oracle Fusion Cloud ERP-inspired General Ledger Allocations:
//! - Allocation pools (cost centers / account ranges to distribute from)
//! - Allocation bases (headcount, revenue, square footage, percentage)
//! - Allocation rules (mapping pools to targets via bases)
//! - Allocation runs (execution generating journal entries)
//! - Step-down allocations
//! - Proportional and fixed percentage distribution methods
//!
//! Oracle Fusion equivalent: General Ledger > Allocations

mod engine;
mod repository;

pub use engine::AllocationEngine;
pub use repository::{AllocationRepository, PostgresAllocationRepository};