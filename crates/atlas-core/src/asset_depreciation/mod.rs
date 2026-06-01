//! Asset Depreciation Module
//!
//! Oracle Fusion Cloud ERP-inspired Fixed Asset Depreciation engine.
//! Implements depreciation calculation methods:
//! - Straight-line depreciation
//! - Declining balance depreciation
//! - Sum-of-years-digits depreciation
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > Depreciation

pub mod engine;
mod repository;

pub use engine::AssetDepreciationEngine;
pub use repository::{AssetDepreciationRepository, PostgresAssetDepreciationRepository};
