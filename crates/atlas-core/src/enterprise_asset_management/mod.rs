//! Enterprise Asset Management (eAM) Module
//!
//! Oracle Fusion Cloud: Maintenance Management / Enterprise Asset Management
//! inspired physical asset management. Provides:
//! - Asset location management
//! - Physical asset definition & hierarchy
//! - Maintenance work orders (corrective, preventive, emergency, inspection)
//! - Preventive maintenance schedules (time-based, meter-based, condition-based)
//! - Material & labor tracking on work orders
//! - Maintenance KPI dashboard (MTBF, MTTR, costs, downtime)
//!
//! Oracle Fusion equivalent: Maintenance Management > Work Orders,
//! Preventive Maintenance, Asset Definition, Maintenance Dashboard

mod repository;
pub mod engine;

pub use engine::EnterpriseAssetManagementEngine;
pub use repository::{AssetManagementRepository, PostgresAssetManagementRepository};
