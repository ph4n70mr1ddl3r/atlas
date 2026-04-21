//! Product Information Management (PIM) Module
//!
//! Oracle Fusion Cloud: Product Hub / Product Information Management
//!
//! Centralized management of product master data including:
//! - Product items with full attribute tracking
//! - Item categories (hierarchical classification)
//! - Item cross-references (GTIN, UPC, supplier part numbers)
//! - New Item Requests (NIR) with approval workflows
//! - Item templates for standardized creation
//! - Item lifecycle phase management
//!
//! Oracle Fusion equivalent: Product Hub > Manage Items

mod engine;
mod repository;

pub use engine::ProductInformationEngine;
pub use repository::{ProductInformationRepository, PostgresProductInformationRepository};
