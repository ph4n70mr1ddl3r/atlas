//! Joint Venture Management Module
//!
//! Oracle Fusion Cloud Financials-inspired Joint Venture Management.
//! Provides joint venture agreements, partner ownership management,
//! AFEs (Authorizations for Expenditure), cost and revenue distributions,
//! and Joint Interest Billing (JIB).
//!
//! Oracle Fusion equivalent: Financials > Joint Venture Management

mod repository;
pub mod engine;

pub use engine::JointVentureEngine;
pub use repository::{JointVentureRepository, PostgresJointVentureRepository};
