//! Accounting Hub Module
//!
//! Oracle Fusion Cloud ERP-inspired Accounting Hub.
//! Processes accounting events from external systems, applies mapping rules,
//! and generates journal entries through the subledger accounting framework.
//!
//! Oracle Fusion equivalent: Financials > Accounting Hub

pub mod engine;
mod repository;

pub use engine::AccountingHubEngine;
pub use repository::{AccountingHubRepository, PostgresAccountingHubRepository};
