//! Period Close Management
//!
//! Oracle Fusion Cloud ERP-inspired General Ledger Period Close.
//! Controls accounting periods, prevents posting to closed periods,
//! tracks subledger close status, and provides a period close checklist.

mod engine;
mod repository;

pub use engine::PeriodCloseEngine;
pub use repository::{PeriodCloseRepository, PostgresPeriodCloseRepository};
