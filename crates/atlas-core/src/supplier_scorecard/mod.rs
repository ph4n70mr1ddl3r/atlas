//! Supplier Scorecard Management Module
//!
//! Oracle Fusion Supplier Portal: Supplier Performance Management.
//! Manages scorecard templates, KPI categories, supplier scorecards,
//! performance reviews, and action items.
//!
//! Oracle Fusion equivalent: Supplier Portal > Performance

mod engine;
mod repository;

pub use engine::SupplierScorecardEngine;
pub use repository::{ScorecardRepository, PostgresScorecardRepository};
