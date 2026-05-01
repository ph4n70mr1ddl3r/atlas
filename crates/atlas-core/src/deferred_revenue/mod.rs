//! Deferred Revenue/Cost Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Deferred Revenue and Cost Management.
//! Manages deferral templates, automated recognition schedules, and amortization.
//!
//! Oracle Fusion equivalent: Financials > Revenue Management > Deferral Schedules

mod repository;
pub mod engine;

pub use engine::DeferredRevenueEngine;
pub use repository::{DeferredRevenueRepository, PostgresDeferredRevenueRepository};
