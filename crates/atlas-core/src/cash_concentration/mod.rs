//! Cash Concentration / Pooling Module
//!
//! Oracle Fusion Cloud ERP-inspired Treasury Cash Concentration & Pooling.
//! Supports physical and notional cash pools, sweep rules, sweep runs,
//! and pool participant management for automated cash concentration.
//!
//! Oracle Fusion equivalent: Treasury > Cash Pooling > Cash Concentration

pub mod engine;
pub mod repository;

pub use engine::CashConcentrationEngine;
pub use repository::{CashConcentrationRepository, PostgresCashConcentrationRepository};
