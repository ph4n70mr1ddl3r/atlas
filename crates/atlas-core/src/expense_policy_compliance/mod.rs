//! Expense Policy Compliance Module
//!
//! Oracle Fusion Cloud ERP-inspired Expense Policy Compliance Engine.
//! Provides configurable expense policy rules, compliance evaluation,
//! violation tracking, and audit scoring for corporate expense management.
//!
//! Oracle Fusion equivalent: Expenses > Policies > Expense Policy Compliance

mod repository;
pub mod engine;

pub use engine::ExpensePolicyComplianceEngine;
pub use repository::{ExpensePolicyComplianceRepository, PostgresExpensePolicyComplianceRepository};
