//! Service Request Management Module
//!
//! Oracle Fusion CX Service-inspired Service Request Management.
//! Provides service request categories, request lifecycle management,
//! assignments, communications, SLA tracking, and resolution.
//!
//! Oracle Fusion equivalent: CX Service > Service Requests

pub mod engine;
mod repository;

pub use engine::ServiceRequestEngine;
pub use repository::{PostgresServiceRequestRepository, ServiceRequestRepository};
