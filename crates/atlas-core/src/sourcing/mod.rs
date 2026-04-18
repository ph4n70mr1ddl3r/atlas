//! Procurement Sourcing Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Procurement Sourcing.
//! Provides sourcing events (RFQ/RFP/RFI), supplier responses/bids,
//! scoring & evaluation, award management, and sourcing templates.
//!
//! Oracle Fusion equivalent: Procurement > Sourcing > Negotiations

mod repository;
pub mod engine;

pub use engine::SourcingEngine;
pub use repository::{SourcingRepository, PostgresSourcingRepository};
