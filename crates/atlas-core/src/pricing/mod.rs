//! Advanced Pricing Management
//!
//! Oracle Fusion Cloud ERP-inspired Advanced Pricing.
//! Provides price lists, tiered pricing, discount rules, charge definitions,
//! pricing strategies, and a price calculation engine.
//!
//! Oracle Fusion equivalent: Order Management > Pricing > Advanced Pricing

mod repository;
pub mod engine;

pub use engine::PricingEngine;
pub use repository::{PricingRepository, PostgresPricingRepository};
