//! Recurring Invoice Management
//!
//! Oracle Fusion Cloud ERP: Financials > Payables > Recurring Invoices
//!
//! Manages recurring AP invoice templates with automatic generation:
//! - Template CRUD with recurrence schedules
//! - Template line management with account distributions
//! - Automatic invoice generation based on schedule
//! - Generation audit trail
//! - Dashboard with upcoming invoices and statistics

pub mod engine;
pub mod repository;

pub use engine::RecurringInvoiceEngine;
pub use repository::{RecurringInvoiceRepository, PostgresRecurringInvoiceRepository};
