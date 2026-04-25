//! Demand Planning / Demand Management Module
//!
//! Oracle Fusion SCM: Demand Management implementation.
//! Manages demand forecast methods, demand schedules (forecasts),
//! schedule lines, historical demand data, forecast consumption,
//! accuracy measurement, and demand planning analytics.
//!
//! Oracle Fusion equivalent: SCM > Demand Management

mod engine;
mod repository;

pub use engine::DemandPlanningEngine;
pub use repository::{DemandPlanningRepository, PostgresDemandPlanningRepository};
