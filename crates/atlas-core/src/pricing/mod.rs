//! Advanced Pricing Management
//!
//! Oracle Fusion Cloud ERP-inspired Advanced Pricing.
//! Provides price lists, tiered pricing, discount rules, charge definitions,
//! pricing strategies, and a price calculation engine.
//!
//! Oracle Fusion equivalent: Order Management > Pricing > Advanced Pricing

pub mod engine;
mod repository;

pub use engine::PricingEngine;
pub use repository::{PostgresPricingRepository, PricingRepository};
