//! Inflation Adjustment Module (IAS 29)
//!
//! Oracle Fusion Cloud ERP-inspired hyperinflationary economy accounting.
//! Manages inflation indices, index rates, and inflation adjustment runs
//! for restating financial statements in hyperinflationary economies
//! per IAS 29 requirements.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Inflation Adjustment

mod repository;
pub mod engine;

pub use engine::InflationAdjustmentEngine;
pub use repository::{InflationAdjustmentRepository, PostgresInflationAdjustmentRepository};
