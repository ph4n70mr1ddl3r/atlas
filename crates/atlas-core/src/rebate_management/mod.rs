//! Rebate Management Module
//!
//! Oracle Fusion Cloud: Trade Management > Rebates
//! Provides:
//! - Rebate agreement management (supplier and customer rebates)
//! - Tiered rebate structures with volume-based pricing
//! - Rebate transaction tracking (qualifying purchases/sales)
//! - Rebate accruals with GL posting
//! - Rebate settlement processing (payments, credit memos)
//! - Rebate analytics dashboard
//!
//! Oracle Fusion equivalent: Trade Management > Rebates > Agreements, Processing, Settlement

mod repository;
pub mod engine;

pub use engine::RebateManagementEngine;
pub use repository::{RebateManagementRepository, PostgresRebateManagementRepository};
