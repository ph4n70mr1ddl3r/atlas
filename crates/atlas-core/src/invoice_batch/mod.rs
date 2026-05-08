//! AP Invoice Batch Processing Module
//!
//! Oracle Fusion Cloud ERP-inspired Invoice Batch Processing.
//! Manages the grouping, validation, approval, and posting of AP invoices
//! in batches with full lifecycle:
//! draft → submitted → approved → posted → cancelled
//!
//! Oracle Fusion equivalent: Financials > Payables > Invoice Batches

pub mod repository;
pub mod engine;

pub use engine::InvoiceBatchEngine;
pub use repository::{InvoiceBatchRepository, PostgresInvoiceBatchRepository};
