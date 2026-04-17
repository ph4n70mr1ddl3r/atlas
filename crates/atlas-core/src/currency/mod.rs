//! Currency Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Multi-Currency Management.
//! Provides currency definitions, daily exchange rates, automatic
//! conversion with triangulation, and unrealized gain/loss calculation.
//!
//! Oracle Fusion equivalent: General Ledger > Currency Rates Manager

mod repository;
pub mod engine;

pub use engine::CurrencyEngine;
pub use repository::{CurrencyRepository, PostgresCurrencyRepository};
