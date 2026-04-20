//! Currency Revaluation Module
//!
//! Oracle Fusion Cloud ERP-inspired Currency Revaluation:
//! - Define revaluation rules specifying accounts, rates, and gain/loss accounts
//! - Execute period-end revaluation runs for foreign currency balances
//! - Generate unrealized gain/loss journal entries
//! - Reverse revaluation entries in the next period
//! - Track revaluation history
//!
//! Oracle Fusion equivalent: General Ledger > Currency Revaluation

mod engine;
mod repository;

pub use engine::CurrencyRevaluationEngine;
pub use repository::{CurrencyRevaluationRepository, PostgresCurrencyRevaluationRepository};