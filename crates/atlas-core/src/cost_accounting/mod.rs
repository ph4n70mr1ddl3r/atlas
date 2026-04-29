//! Cost Accounting Module
//!
//! Oracle Fusion Cost Management / Cost Accounting.
//! Manages product costing with standard, average, FIFO, and LIFO methods.
//! Includes cost books, cost elements, cost profiles, standard costs,
//! cost adjustments, and variance analysis.
//!
//! Oracle Fusion equivalent: Cost Management > Cost Accounting

mod engine;
mod repository;

pub use engine::CostAccountingEngine;
pub use repository::{CostAccountingRepository, PostgresCostAccountingRepository};
