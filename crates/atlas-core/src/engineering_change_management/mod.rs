//! Engineering Change Management (ECM) Module
//!
//! Oracle Fusion Cloud: Product Development > Engineering Change Management
//! Provides:
//! - Engineering Change Types (ECR, ECO, ECN configuration)
//! - Engineering Change Orders with full lifecycle (draft → submitted → in_review → approved/rejected → implemented → closed)
//! - Change lines tracking individual field/BOM changes
//! - Affected items with impact analysis
//! - Multi-level approval workflow
//! - Revision management and supersession chains
//! - ECM dashboard with analytics
//!
//! Oracle Fusion equivalent: Product Development > Engineering Change Management

mod repository;
pub mod engine;

pub use engine::EngineeringChangeEngine;
pub use repository::{EngineeringChangeManagementRepository, PostgresEngineeringChangeManagementRepository};
