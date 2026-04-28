//! Channel Revenue Management (Trade Promotion Management)
//!
//! Oracle Fusion Cloud CX > Channel Revenue Management
//! Provides:
//! - Trade promotion planning, execution, and tracking
//! - Promotion fund management and allocation
//! - Trade claims (billbacks, proof-of-performance, lump sums)
//! - Settlement processing (payments, credit memos, offsets)
//! - Accrual management and trade spend analytics
//! - Promotion ROI and dashboard analytics
//!
//! Oracle Fusion equivalent: CX > Channel Revenue Management > Trade Promotions, Funds, Claims

mod repository;
pub mod engine;

pub use engine::ChannelRevenueEngine;
pub use repository::{ChannelRevenueRepository, PostgresChannelRevenueRepository};
