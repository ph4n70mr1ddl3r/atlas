//! Hedge Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Treasury Hedge Management.
//! Provides derivative instrument tracking, hedge relationship designation,
//! effectiveness testing (IFRS 9 / ASC 815), and hedge documentation
//! for hedge accounting compliance.
//!
//! Oracle Fusion equivalent: Treasury > Hedge Management

pub mod repository;
pub mod engine;

pub use engine::HedgeManagementEngine;
pub use repository::{HedgeManagementRepository, PostgresHedgeManagementRepository};