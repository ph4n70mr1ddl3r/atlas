//! Expense Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Expense Management.
//! Provides expense categories, policies, expense reports with line items,
//! per-diem calculation, mileage calculation, policy validation, and
//! reimbursement processing.
//!
//! Oracle Fusion equivalent: Expenses > Expense Reports, Categories, Policies

pub mod engine;
mod repository;

pub use engine::ExpenseEngine;
pub use repository::{ExpenseRepository, PostgresExpenseRepository};
