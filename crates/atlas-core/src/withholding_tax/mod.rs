//! Withholding Tax Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Withholding Tax Management.
//! Provides withholding tax codes, tax groups, supplier assignments,
//! automatic withholding computation during payment, thresholds,
//! exemptions, and certificate management.
//!
//! Oracle Fusion equivalent: Financials > Payables > Withholding Tax

pub mod engine;
mod repository;

pub use engine::WithholdingTaxEngine;
pub use repository::{PostgresWithholdingTaxRepository, WithholdingTaxRepository};
