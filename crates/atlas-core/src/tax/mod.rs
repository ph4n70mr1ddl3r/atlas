//! Tax Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Tax Management.
//! Provides tax regimes, jurisdictions, tax rates, determination rules,
//! and transaction-level tax calculation with recovery support.
//!
//! Oracle Fusion equivalent: Tax > Tax Configuration and Calculation

pub mod engine;
mod repository;

pub use engine::TaxEngine;
pub use repository::{PostgresTaxRepository, TaxRepository};
