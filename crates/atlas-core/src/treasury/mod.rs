//! Treasury Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Treasury Management.
//! Provides counterparty (bank) management, treasury deal lifecycle
//! (investments, borrowings, FX deals), interest calculations, settlement
//! processing, and maturity tracking.
//!
//! Oracle Fusion equivalent: Financials > Treasury > Deals

pub mod engine;
mod repository;

pub use engine::TreasuryEngine;
pub use repository::{PostgresTreasuryRepository, TreasuryRepository};
