//! Marketing Campaign Management Module
//!
//! Oracle Fusion Cloud CX Marketing implementation.
//! Manages marketing campaigns, campaign types, members, responses,
//! and ROI analytics.
//!
//! Oracle Fusion equivalent: CX Marketing > Campaigns

mod engine;
mod repository;

pub use engine::MarketingEngine;
pub use repository::{MarketingRepository, PostgresMarketingRepository};
