//! Multi-Book Accounting (Secondary Ledgers) Module
//!
//! Oracle Fusion Cloud ERP-inspired Multi-Book Accounting.
//! Provides accounting books (primary and secondary), account mapping rules,
//! book-specific journal entries, automatic journal propagation between books,
//! and multi-GAAP compliance support.
//!
//! Oracle Fusion equivalent: General Ledger > Multi-Book Accounting

pub mod engine;
mod repository;

pub use engine::MultiBookAccountingEngine;
pub use repository::{MultiBookAccountingRepository, PostgresMultiBookAccountingRepository};
