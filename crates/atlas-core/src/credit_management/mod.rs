//! Credit Management Module
//!
//! Oracle Fusion Cloud Credit Management implementation.
//! Manages customer credit profiles, credit scoring, credit limits,
//! credit exposure tracking, credit check rules, credit holds,
//! and credit reviews.
//!
//! Oracle Fusion equivalent: Receivables > Credit Management

mod engine;
mod repository;

pub use engine::CreditManagementEngine;
pub use repository::{CreditManagementRepository, PostgresCreditManagementRepository};
