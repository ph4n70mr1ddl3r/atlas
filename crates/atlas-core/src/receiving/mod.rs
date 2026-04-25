//! Receiving Management Module
//!
//! Oracle Fusion Cloud SCM-inspired Receiving Management.
//! Provides receiving locations, receipt headers and lines,
//! inspection management, delivery/putaway, and return-to-supplier processing.
//!
//! Oracle Fusion equivalent: SCM > Receiving > Receiving

mod repository;
pub mod engine;

pub use engine::ReceivingEngine;
pub use repository::{ReceivingRepository, PostgresReceivingRepository};
