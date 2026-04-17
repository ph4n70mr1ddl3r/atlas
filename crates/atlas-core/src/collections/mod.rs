//! Collections & Credit Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Collections Management.
//! Provides customer credit profiles, credit limits, credit scoring,
//! receivables aging analysis, collection strategies, collection cases,
//! customer interactions, promise-to-pay tracking, dunning campaigns,
//! dunning letters, and write-off management.
//!
//! Oracle Fusion equivalent: Financials > Collections > Collections Management

mod repository;
pub mod engine;

pub use engine::CollectionsEngine;
pub use repository::{CollectionsRepository, PostgresCollectionsRepository};
