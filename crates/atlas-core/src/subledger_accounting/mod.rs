//! Subledger Accounting Module
//!
//! Oracle Fusion Cloud ERP-inspired Subledger Accounting.
//! Bridges subledger transactions (AP, AR, Expenses, etc.) to the General Ledger
//! through accounting methods, derivation rules, journal entry generation,
//! posting, and transfer to GL.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Subledger Accounting

mod repository;
pub mod engine;

pub use engine::SubledgerAccountingEngine;
pub use repository::{SubledgerAccountingRepository, PostgresSubledgerAccountingRepository};