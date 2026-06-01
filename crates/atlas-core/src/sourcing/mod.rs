//! Procurement Sourcing Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Procurement Sourcing.
//! Provides sourcing events (RFQ/RFP/RFI), supplier responses/bids,
//! scoring & evaluation, award management, and sourcing templates.
//!
//! Oracle Fusion equivalent: Procurement > Sourcing > Negotiations

pub mod engine;
mod repository;

pub use engine::SourcingEngine;
pub use repository::{PostgresSourcingRepository, SourcingRepository};
