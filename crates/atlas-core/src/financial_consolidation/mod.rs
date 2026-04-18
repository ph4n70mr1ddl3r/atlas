//! Financial Consolidation Module
//!
//! Oracle Fusion Cloud ERP-inspired Financial Consolidation.
//! Manages consolidation of financial statements across multiple legal entities
//! and business units, including currency translation, intercompany eliminations,
//! minority interest calculations, consolidation adjustments, and equity elimination.
//!
//! Oracle Fusion equivalent: General Ledger > Financial Consolidation

mod repository;
pub mod engine;

pub use engine::FinancialConsolidationEngine;
pub use repository::{FinancialConsolidationRepository, PostgresFinancialConsolidationRepository};
