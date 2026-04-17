//! Tax Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Tax Management.
//! Provides tax regimes, jurisdictions, tax rates, determination rules,
//! and transaction-level tax calculation with recovery support.
//!
//! Oracle Fusion equivalent: Tax > Tax Configuration and Calculation

mod repository;
pub mod engine;

pub use engine::TaxEngine;
pub use repository::{TaxRepository, PostgresTaxRepository};
