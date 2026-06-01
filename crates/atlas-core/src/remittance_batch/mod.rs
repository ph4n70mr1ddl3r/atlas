//! Remittance Batch Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Remittance Batches.
//! Groups receipts into batches for bank deposit processing with full lifecycle:
//! draft → approved → formatted → transmitted → confirmed → settled
//!
//! Oracle Fusion equivalent: Financials > Receivables > Receipts > Automatic Receipts > Remittance Batches

pub mod engine;
pub mod repository;

pub use engine::RemittanceBatchEngine;
pub use repository::{PostgresRemittanceBatchRepository, RemittanceBatchRepository};
