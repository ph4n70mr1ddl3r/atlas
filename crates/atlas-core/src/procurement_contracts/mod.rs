//! Procurement Contracts Module
//!
//! Oracle Fusion Cloud ERP-inspired Procurement Contracts Management.
//! Manages contract types, procurement contracts, contract lines,
//! milestones/deliverables, renewals, spend tracking, and the full
//! contract lifecycle.
//!
//! Oracle Fusion equivalent: SCM > Procurement > Contracts

mod repository;
pub mod engine;

pub use engine::ProcurementContractEngine;
pub use repository::{ProcurementContractRepository, PostgresProcurementContractRepository};
