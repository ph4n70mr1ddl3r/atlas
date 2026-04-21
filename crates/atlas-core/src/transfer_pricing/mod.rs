//! Transfer Pricing Module
//!
//! Oracle Fusion Cloud Financials > Transfer Pricing implementation.
//! Manages intercompany transfer price policies, transactions,
//! benchmarking analyses, and BEPS/OECD documentation for tax compliance.
//!
//! Oracle Fusion equivalent: Financials > Transfer Pricing > Policies > Transactions > Benchmarking

mod engine;
mod repository;

pub use engine::TransferPricingEngine;
pub use repository::{TransferPricingRepository, PostgresTransferPricingRepository};
