//! Descriptive Flexfields Module
//!
//! Oracle Fusion Cloud ERP-inspired Descriptive Flexfields (DFF).
//! Allows administrators to add custom configurable fields to any entity
//! at runtime, with value set validation and context-sensitive segments.
//!
//! Key capabilities:
//! - Define flexfields on any entity with global and context-sensitive segments
//! - Value sets with validation (independent, dependent, table, format-only)
//! - Context-sensitive segments: different custom fields based on context value
//! - Segment ordering, required/optional, default values
//! - Full validation of segment values against configured value sets
//! - CRUD for flexfield data stored alongside entity records
//!
//! Oracle Fusion equivalent: Application Extensions > Flexfields > Descriptive

mod repository;
pub mod engine;

pub use engine::DescriptiveFlexfieldEngine;
pub use repository::{DescriptiveFlexfieldRepository, PostgresDescriptiveFlexfieldRepository};
