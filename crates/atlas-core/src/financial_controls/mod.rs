//! Advanced Financial Controls Module
//!
//! Oracle Fusion Cloud ERP-inspired Advanced Financial Controls.
//! Continuous monitoring, anomaly detection, policy violation tracking,
//! and automated alert management for financial transactions.
//!
//! Oracle Fusion equivalent: Financials > Advanced Controls

mod repository;
pub mod engine;

pub use engine::FinancialControlsEngine;
pub use repository::{FinancialControlsRepository, PostgresFinancialControlsRepository};
