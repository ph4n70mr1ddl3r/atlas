//! Intercompany Transactions Module
//!
//! Oracle Fusion Cloud ERP-inspired Intercompany Management.
//! Manages financial transactions between legal entities / business units
//! within the same enterprise, including:
//! - Intercompany transaction batches
//! - Individual intercompany transactions
//! - Automatic due-to/due-from balancing
//! - Settlement tracking and netting
//! - Outstanding balance monitoring
//!
//! Oracle Fusion equivalent: Intercompany > Intercompany Transactions

mod repository;
pub mod engine;

pub use engine::IntercompanyEngine;
pub use repository::{IntercompanyRepository, PostgresIntercompanyRepository};
