//! Payment Process Request Module
//!
//! Oracle Fusion Cloud ERP-inspired Payment Process Request (PPR).
//! Manages automated batch payment processing with full lifecycle:
//! draft → submitted → `selection_complete` → formatted → confirmed → cancelled
//!
//! Oracle Fusion equivalent: Financials > Payables > Payment Process Requests

pub mod engine;
pub mod repository;

pub use engine::PaymentProcessRequestEngine;
pub use repository::{PaymentProcessRequestRepository, PostgresPaymentProcessRequestRepository};
