//! Recruiting Management Module
//!
//! Oracle Fusion HCM: Recruiting implementation.
//! Manages job requisitions, candidates, job applications,
//! interviews, job offers, and recruiting dashboard analytics.
//!
//! Oracle Fusion equivalent: HCM > Recruiting

mod engine;
mod repository;

pub use engine::RecruitingEngine;
pub use repository::{RecruitingRepository, PostgresRecruitingRepository};
