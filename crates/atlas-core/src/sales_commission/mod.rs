//! Sales Commission Management
//!
//! Oracle Fusion Cloud ERP-inspired Incentive Compensation module.
//! Provides sales representative management, commission plans with tiered rates,
//! quota tracking, commission transaction crediting, and payout processing.
//!
//! Oracle Fusion equivalent: Incentive Compensation > Compensation Plans > Payouts

mod repository;
pub mod engine;

pub use engine::SalesCommissionEngine;
pub use repository::{SalesCommissionRepository, PostgresSalesCommissionRepository};
