//! Accounts Receivable Module
//!
//! Oracle Fusion Cloud ERP-inspired Receivables management.
//! Manages customer transactions, receipts, credit memos, and adjustments.
//!
//! Oracle Fusion equivalent: Financials > Receivables

mod repository;
pub mod engine;

pub use engine::AccountsReceivableEngine;
pub use repository::{AccountsReceivableRepository, PostgresAccountsReceivableRepository};
