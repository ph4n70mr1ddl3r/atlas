//! General Ledger Module
//!
//! Oracle Fusion Cloud ERP-inspired General Ledger management.
//! Manages chart of accounts, journal entries, posting, and trial balance.
//!
//! Oracle Fusion equivalent: Financials > General Ledger

mod repository;
pub mod engine;

pub use engine::GeneralLedgerEngine;
pub use repository::{GeneralLedgerRepository, PostgresGeneralLedgerRepository};
