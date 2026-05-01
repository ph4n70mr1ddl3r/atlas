//! Impairment Management Module (IAS 36 / ASC 360)
//!
//! Oracle Fusion Cloud ERP-inspired asset impairment management.
//! Manages impairment indicators, impairment tests (value-in-use and fair value),
//! cash flow projections, and impairment recognition for fixed assets.
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > Impairment Management

mod repository;
pub mod engine;

pub use engine::ImpairmentManagementEngine;
pub use repository::{ImpairmentManagementRepository, PostgresImpairmentManagementRepository};
