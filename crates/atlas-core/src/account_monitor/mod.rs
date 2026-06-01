//! Account Monitor & Balance Inquiry Module
//!
//! Oracle Fusion General Ledger > Account Monitor and Balance Inquiry.
//! Provides:
//! - Account group management (define which GL accounts to monitor)
//! - Balance snapshot capture (real-time GL balance data)
//! - Threshold-based alerting (warning/critical when balances deviate)
//! - Period-over-period comparisons
//! - Saved balance inquiry configurations
//!
//! Oracle Fusion equivalent: General Ledger > Journals > Account Monitor

pub mod engine;
mod repository;

pub use engine::AccountMonitorEngine;
pub use repository::{AccountMonitorRepository, PostgresAccountMonitorRepository};
