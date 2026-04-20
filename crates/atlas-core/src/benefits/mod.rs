//! Benefits Administration Module
//!
//! Oracle Fusion Cloud HCM-inspired Benefits Administration.
//! Provides benefits plans, coverage tiers, employee enrollments,
//! qualifying life events, and payroll deduction management.
//!
//! Oracle Fusion equivalent: Benefits > Benefits Plans, Enrollments, Coverage

mod repository;
pub mod engine;

pub use engine::BenefitsEngine;
pub use repository::{BenefitsRepository, PostgresBenefitsRepository};
