//! Receiving Management Module
//!
//! Oracle Fusion Cloud SCM-inspired Receiving Management.
//! Provides receiving locations, receipt headers and lines,
//! inspection management, delivery/putaway, and return-to-supplier processing.
//!
//! Oracle Fusion equivalent: SCM > Receiving > Receiving

pub mod engine;
mod repository;

pub use engine::ReceivingEngine;
pub use repository::{PostgresReceivingRepository, ReceivingRepository};
