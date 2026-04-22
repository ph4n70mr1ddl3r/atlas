//! Manufacturing Execution Module
//!
//! Oracle Fusion Cloud SCM > Manufacturing implementation.
//! Manages work definitions (BOMs + Routings), work orders,
//! operations, material requirements, and production completions.
//!
//! Oracle Fusion equivalent: SCM > Manufacturing > Work Definitions, Work Orders

mod repository;
pub mod engine;

pub use engine::ManufacturingEngine;
pub use repository::{ManufacturingRepository, PostgresManufacturingRepository};
