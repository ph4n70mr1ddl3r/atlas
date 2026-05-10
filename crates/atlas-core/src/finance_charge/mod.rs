//! Finance Charge Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Finance Charge Management.
//! Manages automatic assessment of late payment charges on overdue customer invoices.
//!
//! Features:
//! - Finance charge term definitions (percentage, flat fee, tiered)
//! - Finance charge assessment runs (batch processing of overdue invoices)
//! - Finance charge invoice generation
//! - Full lifecycle: draft → submitted → approved → applied → cancelled
//!
//! Oracle Fusion equivalent: Financials > Receivables > Finance Charges

pub mod repository;
pub mod engine;

pub use engine::FinanceChargeEngine;
pub use repository::{FinanceChargeRepository, PostgresFinanceChargeRepository};
