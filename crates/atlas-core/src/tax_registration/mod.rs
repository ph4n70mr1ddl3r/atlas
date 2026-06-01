//! Tax Registration Module
//!
//! Oracle Fusion Cloud ERP-inspired tax registration management.
//! Manages taxpayer identification numbers (TIN, VAT, GST, EIN, etc.)
//! for first-party legal entities and third-party organizations across
//! tax jurisdictions, with validation, status tracking, and compliance.
//!
//! Oracle Fusion equivalent: Financials > Tax > Tax Registrations

pub mod engine;
mod repository;

pub use engine::TaxRegistrationEngine;
pub use repository::{PostgresTaxRegistrationRepository, TaxRegistrationRepository};
