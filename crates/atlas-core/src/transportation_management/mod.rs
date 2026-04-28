//! Transportation Management
//!
//! Oracle Fusion Cloud SCM > Transportation Management
//! Provides:
//! - Carrier management (parcel, LTL, FTL, air, ocean, rail)
//! - Carrier service level management
//! - Transportation lane/route management (origin-destination pairs)
//! - Shipment lifecycle (draft → booked → picked_up → in_transit → delivered)
//! - Multi-stop shipment planning
//! - Shipment line item tracking
//! - Real-time shipment tracking events
//! - Freight rate management and cost calculation
//! - Transportation analytics dashboard
//!
//! Oracle Fusion equivalent: SCM > Transportation Management

mod repository;
pub mod engine;

pub use engine::TransportationManagementEngine;
pub use repository::{TransportationManagementRepository, PostgresTransportationManagementRepository};
