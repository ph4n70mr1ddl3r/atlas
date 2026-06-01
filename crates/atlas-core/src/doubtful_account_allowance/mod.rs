//! Doubtful Account Allowance Module
//!
//! Oracle Fusion Cloud ERP-inspired Allowance for Doubtful Accounts management.
//! Manages provision policies, aging bucket definitions, provision runs,
//! and provision history for AR bad debt allowance calculation.
//!
//! Oracle Fusion equivalent: Receivables > Collections > Allowance for Doubtful Accounts

pub mod engine;
mod repository;

pub use engine::DoubtfulAccountAllowanceEngine;
pub use repository::{
    DoubtfulAccountAllowanceRepository, PostgresDoubtfulAccountAllowanceRepository,
};
