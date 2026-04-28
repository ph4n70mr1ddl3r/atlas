//! Product Configurator (Configure-to-Order)
//!
//! Oracle Fusion Cloud SCM > Product Management > Configurator
//! Provides:
//! - Configurable product model definitions
//! - Configuration features and option management
//! - Configuration rules (compatibility, incompatibility, requirements)
//! - Configuration instance lifecycle (create, validate, submit, approve)
//! - Pricing aggregation from option selections
//! - Configurator dashboard analytics
//!
//! Oracle Fusion equivalent: SCM > Product Management > Configurator

mod repository;
pub mod engine;

pub use engine::ProductConfiguratorEngine;
pub use repository::{ProductConfiguratorRepository, PostgresProductConfiguratorRepository};
