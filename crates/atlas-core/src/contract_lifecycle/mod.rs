//! Contract Lifecycle Management Module
//!
//! Oracle Fusion Enterprise Contracts-inspired contract management. Provides:
//! - Contract type definitions
//! - Clause library management
//! - Contract template management with clause composition
//! - Contract lifecycle (draft → active → completed/terminated)
//! - Contract parties, milestones, and deliverables
//! - Contract amendments and change management
//! - Risk assessment and scoring
//!
//! Oracle Fusion equivalent: Enterprise Contracts > Contract Management

mod repository;
pub mod engine;

pub use engine::ContractLifecycleEngine;
pub use repository::{ContractLifecycleRepository, PostgresContractLifecycleRepository};
