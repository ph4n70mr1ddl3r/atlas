//! Warehouse Management
//!
//! Oracle Fusion Cloud Warehouse Management
//!
//! Manages warehouse operations including:
//! - Warehouse & zone configuration
//! - Put-away rules for intelligent item placement
//! - Warehouse tasks (pick, pack, put-away, load, receive)
//! - Pick wave management for batch picking

mod engine;
mod repository;

pub use engine::WarehouseManagementEngine;
pub use repository::{WarehouseManagementRepository, PostgresWarehouseManagementRepository};
