//! Financial Reporting Module (Oracle Fusion GL > Financial Reporting Center)
//!
//! Oracle Fusion Cloud ERP-inspired Financial Reporting.
//! Provides report templates, report rows, report columns, report generation,
//! trial balance, income statement, balance sheet, and custom financial reports.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Financial Reporting Center

mod repository;
pub mod engine;

pub use engine::FinancialReportingEngine;
pub use repository::{FinancialReportingRepository, PostgresFinancialReportingRepository};
