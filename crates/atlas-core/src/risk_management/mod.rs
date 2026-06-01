//! Risk Management & Internal Controls Module
//!
//! Oracle Fusion Cloud GRC (Governance, Risk, and Compliance) / Advanced Controls
//! inspired risk management. Provides:
//! - Risk category management
//! - Risk register (identification & assessment)
//! - Control registry (preventive, detective, corrective controls)
//! - Risk-control mappings
//! - Control testing & certification
//! - Issue & remediation tracking
//! - Risk dashboard with heat map scoring
//!
//! Oracle Fusion equivalent: GRC > Risk Manager, Advanced Controls, Issue Management

pub mod engine;
mod repository;

pub use engine::RiskManagementEngine;
pub use repository::{PostgresRiskManagementRepository, RiskManagementRepository};
