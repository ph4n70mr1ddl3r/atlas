//! Profitability Analysis Module
//!
//! Oracle Fusion Cloud ERP-inspired Profitability Analysis.
//! Manages segment-based profitability: revenue, COGS, margins (gross,
//! operating, net), period-over-period comparisons, and dashboards.
//!
//! Oracle Fusion equivalent: Financials > Profitability Analysis

pub mod repository;
pub mod engine;

pub use engine::ProfitabilityAnalysisEngine;
pub use repository::{ProfitabilityAnalysisRepository, PostgresProfitabilityAnalysisRepository};
