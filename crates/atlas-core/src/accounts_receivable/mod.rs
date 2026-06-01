//! Accounts Receivable Module
//!
//! Oracle Fusion Cloud ERP-inspired Receivables management.
//! Manages customer transactions, receipts, credit memos, and adjustments.
//!
//! Oracle Fusion equivalent: Financials > Receivables

pub mod engine;
mod repository;

pub use engine::AccountsReceivableEngine;
pub use repository::{AccountsReceivableRepository, PostgresAccountsReceivableRepository};
