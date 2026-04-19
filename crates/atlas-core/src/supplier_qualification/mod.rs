//! Supplier Qualification Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Supplier Qualification Management.
//! Provides qualification area management, questionnaire design, supplier qualification
//! initiatives, response collection & evaluation, scoring, and certification tracking.
//!
//! Oracle Fusion equivalent: Procurement > Supplier Qualification > Initiatives

mod repository;
pub mod engine;

pub use engine::SupplierQualificationEngine;
pub use repository::{SupplierQualificationRepository, PostgresSupplierQualificationRepository};
