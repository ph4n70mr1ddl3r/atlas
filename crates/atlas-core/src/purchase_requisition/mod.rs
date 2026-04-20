//! Purchase Requisitions Module
//!
//! Oracle Fusion Cloud ERP-inspired Purchase Requisitions:
//! - Create and manage purchase requisitions (internal requests for goods/services)
//! - Multi-line requisitions with accounting distributions
//! - Configurable approval workflow (submit → approve/reject)
//! - AutoCreate: convert approved requisitions into purchase orders
//! - Requisition prioritization and categorization
//!
//! Oracle Fusion equivalent: Self-Service Procurement > Requisitions

mod engine;
mod repository;

pub use engine::PurchaseRequisitionEngine;
pub use repository::{PurchaseRequisitionRepository, PostgresPurchaseRequisitionRepository};