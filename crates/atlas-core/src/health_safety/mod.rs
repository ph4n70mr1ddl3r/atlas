//! Workplace Health & Safety (EHS) Module
//!
//! Oracle Fusion Cloud: Environment, Health, and Safety
//! Provides:
//! - Safety incident tracking (injuries, near-misses, property damage, environmental releases)
//! - Hazard identification and risk assessment
//! - Safety inspections and audits
//! - Corrective and Preventive Actions (CAPA)
//! - OSHA compliance reporting
//! - Health & Safety dashboard with analytics
//!
//! Oracle Fusion equivalent: EHS > Incident Management, Hazard Management,
//!   Inspection Management, Corrective Action Management

mod repository;
pub mod engine;

pub use engine::HealthSafetyEngine;
pub use repository::{HealthSafetyRepository, PostgresHealthSafetyRepository};
