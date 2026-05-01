//! Tax Reporting Module
//!
//! Oracle Fusion Cloud ERP-inspired tax return filing and compliance.
//! Manages tax return templates, filing schedules, return preparation,
//! and payment tracking for VAT/GST/sales tax/corporate income tax.
//!
//! Oracle Fusion equivalent: Financials > Tax > Tax Reporting

mod repository;
pub mod engine;

pub use engine::TaxReportingEngine;
pub use repository::{TaxReportingRepository, PostgresTaxReportingRepository};
