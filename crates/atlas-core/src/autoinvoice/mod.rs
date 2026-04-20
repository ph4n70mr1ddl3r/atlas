//! AutoInvoice Module
//!
//! Oracle Fusion Cloud Receivables AutoInvoice implementation.
//! Automatically creates AR invoices from imported transaction data
//! with validation rules, grouping rules, and line ordering.
//!
//! Oracle Fusion equivalent: Receivables > AutoInvoice

mod engine;
mod repository;

pub use engine::AutoInvoiceEngine;
pub use repository::{AutoInvoiceRepository, PostgresAutoInvoiceRepository};
