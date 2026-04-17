//! Encumbrance Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Encumbrance Management.
//! Tracks financial commitments before actual expenditure:
//! - Requisitions → Preliminary encumbrances
//! - Purchase Orders → Encumbrances (commitments)
//! - Invoices → Partial/Full liquidation of encumbrances
//! - Contracts → Long-term commitment tracking
//!
//! Provides budgetary control by reserving funds against budgets.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Encumbrance Management

mod repository;
pub mod engine;

pub use engine::EncumbranceEngine;
pub use repository::{EncumbranceRepository, PostgresEncumbranceRepository};
