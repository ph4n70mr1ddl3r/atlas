//! AP/AR Netting Module
//!
//! Oracle Fusion Cloud ERP-inspired netting of payables and receivables.
//! Allows organizations to settle AP invoices and AR transactions with the
//! same trading partner by netting amounts, reducing cash movement.
//!
//! Oracle Fusion equivalent: Financials > Netting

pub mod engine;
mod repository;

pub use engine::NettingEngine;
pub use repository::{NettingRepository, PostgresNettingRepository};
