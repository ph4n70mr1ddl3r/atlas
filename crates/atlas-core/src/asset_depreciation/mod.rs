//! Asset Depreciation Module
//!
//! Oracle Fusion Cloud ERP-inspired Fixed Asset Depreciation engine.
//! Implements depreciation calculation methods:
//! - Straight-line depreciation
//! - Declining balance depreciation
//! - Sum-of-years-digits depreciation
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > Depreciation

mod repository;
pub mod engine;

pub use engine::AssetDepreciationEngine;
pub use repository::{AssetDepreciationRepository, PostgresAssetDepreciationRepository};
