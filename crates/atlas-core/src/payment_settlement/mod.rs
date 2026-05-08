//! Payment Settlement & Clearing Module
//!
//! Oracle Fusion Cloud ERP-inspired Payment Settlement.
//! Manages the settlement of payments against invoices with full lifecycle:
//! draft → submitted → approved → settled → cancelled
//!
//! Oracle Fusion equivalent: Financials > Payables > Settlement

pub mod repository;
pub mod engine;

pub use engine::PaymentSettlementEngine;
pub use repository::{PaymentSettlementRepository, PostgresPaymentSettlementRepository};
