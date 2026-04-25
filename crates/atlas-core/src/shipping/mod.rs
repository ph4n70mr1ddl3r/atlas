//! Shipping Execution Module
//!
//! Oracle Fusion SCM: Shipping Execution implementation.
//! Manages carriers, shipping methods, shipments, shipment lines,
//! packing slips, and shipping dashboard analytics.
//!
//! Oracle Fusion equivalent: SCM > Shipping Execution

mod engine;
mod repository;

pub use engine::ShippingEngine;
pub use repository::{ShippingRepository, PostgresShippingRepository};
