//! Financial Statement Generation Module
//!
//! Oracle Fusion Cloud ERP-inspired financial statement generation:
//! - Balance Sheet (Statement of Financial Position)
//! - Income Statement (Profit & Loss)
//! - Cash Flow Statement (indirect method)
//! - Trial Balance
//! - Statement of Changes in Equity
//!
//! Oracle Fusion equivalent: General Ledger > Financial Reporting Center

pub mod engine;
mod repository;

pub use engine::FinancialStatementEngine;
pub use repository::{FinancialStatementRepository, PostgresFinancialStatementRepository};
