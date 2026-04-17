//! Revenue Recognition Module (ASC 606 / IFRS 15)
//!
//! Oracle Fusion Cloud ERP-inspired Revenue Management.
//! Implements the five-step revenue recognition model:
//! 1. Identify the contract with a customer
//! 2. Identify the performance obligations
//! 3. Determine the transaction price
//! 4. Allocate the transaction price to performance obligations
//! 5. Recognize revenue when (or as) obligations are satisfied
//!
//! Oracle Fusion equivalent: Financials > Revenue Management

mod repository;
pub mod engine;

pub use engine::RevenueEngine;
pub use repository::{RevenueRepository, PostgresRevenueRepository};
