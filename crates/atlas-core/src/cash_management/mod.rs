//! Cash Position & Cash Forecasting Module
//!
//! Oracle Fusion Cloud ERP-inspired Treasury Management.
//! Provides real-time cash positions across bank accounts, cash flow forecasting
//! with configurable time buckets, forecast templates, and forecast sources.
//!
//! Oracle Fusion equivalent: Financials > Treasury > Cash Management

mod repository;
pub mod engine;

pub use engine::CashManagementEngine;
pub use repository::{CashManagementRepository, PostgresCashManagementRepository};
