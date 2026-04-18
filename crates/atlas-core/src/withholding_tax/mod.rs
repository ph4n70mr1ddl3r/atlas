//! Withholding Tax Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Withholding Tax Management.
//! Provides withholding tax codes, tax groups, supplier assignments,
//! automatic withholding computation during payment, thresholds,
//! exemptions, and certificate management.
//!
//! Oracle Fusion equivalent: Financials > Payables > Withholding Tax

mod repository;
pub mod engine;

pub use engine::WithholdingTaxEngine;
pub use repository::{WithholdingTaxRepository, PostgresWithholdingTaxRepository};
