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

mod repository;
pub mod engine;

pub use engine::FinancialStatementEngine;
pub use repository::{FinancialStatementRepository, PostgresFinancialStatementRepository};
