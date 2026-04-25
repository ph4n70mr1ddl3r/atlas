//! Service Request Management Module
//!
//! Oracle Fusion CX Service-inspired Service Request Management.
//! Provides service request categories, request lifecycle management,
//! assignments, communications, SLA tracking, and resolution.
//!
//! Oracle Fusion equivalent: CX Service > Service Requests

mod repository;
pub mod engine;

pub use engine::ServiceRequestEngine;
pub use repository::{ServiceRequestRepository, PostgresServiceRequestRepository};
