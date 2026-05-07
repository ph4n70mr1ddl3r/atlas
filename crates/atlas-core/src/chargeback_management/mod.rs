//! Chargeback Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Chargeback Management.
//! Manages customer-initiated payment deductions with full lifecycle:
//! open → under_review → accepted → rejected → written_off
//!
//! Oracle Fusion equivalent: Financials > Receivables > Chargebacks

pub mod repository;
pub mod engine;

pub use engine::ChargebackManagementEngine;
pub use repository::{ChargebackManagementRepository, PostgresChargebackManagementRepository};
