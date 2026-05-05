//! Tax Registration Module
//!
//! Oracle Fusion Cloud ERP-inspired tax registration management.
//! Manages taxpayer identification numbers (TIN, VAT, GST, EIN, etc.)
//! for first-party legal entities and third-party organizations across
//! tax jurisdictions, with validation, status tracking, and compliance.
//!
//! Oracle Fusion equivalent: Financials > Tax > Tax Registrations

mod repository;
pub mod engine;

pub use engine::TaxRegistrationEngine;
pub use repository::{TaxRegistrationRepository, PostgresTaxRegistrationRepository};
