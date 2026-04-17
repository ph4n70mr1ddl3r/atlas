//! Budget Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Budget Management.
//! Provides budget definitions, budget versions with approval workflow,
//! budget lines (by account, period, department), budget vs. actuals
//! variance reporting, budget transfers, and budget controls.
//!
//! Oracle Fusion equivalent: General Ledger > Budgets

mod repository;
pub mod engine;

pub use engine::BudgetEngine;
pub use repository::{BudgetRepository, PostgresBudgetRepository};
