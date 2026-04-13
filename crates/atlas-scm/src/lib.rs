//! Atlas SCM - Supply Chain Management
//! 
//! Provides inventory management, supplier management, purchase and sales
//! order processing, and warehouse operations.

pub mod entities;
pub mod services;

pub use services::{InventoryService, SupplierService};
