//! Treasury Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Treasury Management.
//! Provides counterparty (bank) management, treasury deal lifecycle
//! (investments, borrowings, FX deals), interest calculations, settlement
//! processing, and maturity tracking.
//!
//! Oracle Fusion equivalent: Financials > Treasury > Deals

mod repository;
pub mod engine;

pub use engine::TreasuryEngine;
pub use repository::{TreasuryRepository, PostgresTreasuryRepository};
