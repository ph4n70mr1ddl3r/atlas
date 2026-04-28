//! Sustainability & ESG Management Module
//!
//! Oracle Fusion Cloud: Sustainability / Environmental Accounting and Reporting
//! Provides:
//! - Facility tracking for environmental footprint
//! - GHG emissions tracking (Scope 1, 2, 3) with emission factors
//! - Energy consumption, water usage, and waste tracking
//! - ESG metrics and readings (GRI, SASB, TCFD aligned)
//! - Sustainability goals with progress tracking
//! - Carbon offset management
//! - Sustainability dashboard with analytics
//!
//! Oracle Fusion equivalent: Sustainability > Environmental Accounting, ESG Reporting

mod repository;
pub mod engine;

pub use engine::SustainabilityEngine;
pub use repository::{SustainabilityRepository, PostgresSustainabilityRepository};
