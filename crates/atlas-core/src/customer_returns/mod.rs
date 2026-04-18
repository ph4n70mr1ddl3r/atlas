//! Customer Returns Management / Return Material Authorization (RMA)
//!
//! Oracle Fusion Cloud ERP-inspired Customer Returns Management.
//! Provides return reason codes, RMA creation and lifecycle management,
//! return receipt and inspection, credit memo generation, and returns analytics.
//!
//! Oracle Fusion equivalent: Order Management > Returns > Return Material Authorization

mod repository;
pub mod engine;

pub use engine::CustomerReturnsEngine;
pub use repository::{CustomerReturnsRepository, PostgresCustomerReturnsRepository};
