//! Fixed Assets Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Fixed Assets Management.
//! Provides asset categories, asset books (corporate/tax), asset registration
//! with depreciation parameters, depreciation calculation methods
//! (straight-line, declining balance, sum-of-years-digits), asset lifecycle
//! management, asset transfers, and asset retirement with gain/loss.
//!
//! Oracle Fusion equivalent: Fixed Assets

mod repository;
pub mod engine;

pub use engine::FixedAssetEngine;
pub use repository::{FixedAssetRepository, PostgresFixedAssetRepository};
