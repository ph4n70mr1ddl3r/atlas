//! Bank Reconciliation
//!
//! Oracle Fusion Cloud ERP-inspired Cash Management Bank Reconciliation.
//! Manages bank accounts, bank statement import, auto-matching of statement
//! lines to system transactions (AP payments, AR receipts, GL entries),
//! manual matching/unmatching, and reconciliation summary tracking.

mod engine;
mod repository;

pub use engine::ReconciliationEngine;
pub use repository::{ReconciliationRepository, PostgresReconciliationRepository};
