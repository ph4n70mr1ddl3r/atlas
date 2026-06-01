//! Manufacturing Execution Module
//!
//! Oracle Fusion Cloud SCM > Manufacturing implementation.
//! Manages work definitions (BOMs + Routings), work orders,
//! operations, material requirements, and production completions.
//!
//! Oracle Fusion equivalent: SCM > Manufacturing > Work Definitions, Work Orders

pub mod engine;
mod repository;

pub use engine::ManufacturingEngine;
pub use repository::{ManufacturingRepository, PostgresManufacturingRepository};
