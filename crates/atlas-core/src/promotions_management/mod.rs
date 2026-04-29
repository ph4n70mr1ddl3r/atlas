//! Promotions Management Module
//!
//! Oracle Fusion Trade Management > Trade Promotion.
//! Manages trade promotions, promotional offers, fund allocation,
//! claims processing, and ROI analysis.
//!
//! Oracle Fusion equivalent: Trade Management > Trade Promotion

mod engine;
mod repository;

pub use engine::PromotionsManagementEngine;
pub use repository::{PromotionsManagementRepository, PostgresPromotionsManagementRepository};
