//! Inventory Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Inventory Management.
//! Manages inventory organizations, items, subinventories, locators,
//! on-hand balances, inventory transactions, and cycle counting.
//!
//! Oracle Fusion equivalent: SCM > Inventory Management

mod repository;
pub mod engine;

pub use engine::InventoryEngine;
pub use repository::{InventoryRepository, PostgresInventoryRepository};
