//! Order Management Module
//!
//! Oracle Fusion Cloud SCM > Order Management implementation.
//! Manages sales orders, order lines, fulfillment orchestration,
//! order holds, shipments, and backorder management.
//!
//! Oracle Fusion equivalent: SCM > Order Management > Sales Orders > Fulfillment > Holds

mod engine;
mod repository;

pub use engine::OrderManagementEngine;
pub use repository::{OrderManagementRepository, PostgresOrderManagementRepository};
