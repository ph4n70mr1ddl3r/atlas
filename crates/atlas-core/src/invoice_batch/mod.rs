//! AP Invoice Batch Processing Module
//!
//! Oracle Fusion Cloud ERP-inspired Invoice Batch Processing.
//! Manages the grouping, validation, approval, and posting of AP invoices
//! in batches with full lifecycle:
//! draft → submitted → approved → posted → cancelled
//!
//! Oracle Fusion equivalent: Financials > Payables > Invoice Batches

pub mod engine;
pub mod repository;

pub use engine::InvoiceBatchEngine;
pub use repository::{InvoiceBatchRepository, PostgresInvoiceBatchRepository};
