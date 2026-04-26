//! KPI & Embedded Analytics Module
//!
//! Oracle Fusion OTBI (Oracle Transactional Business Intelligence)-inspired
//! KPI management and embedded analytics. Provides:
//! - KPI definitions with targets and thresholds
//! - KPI data point tracking (time-series)
//! - Dashboard management with configurable widgets
//! - KPI evaluation engine
//!
//! Oracle Fusion equivalent: Analytics > KPI Library, Dashboards, OTBI

mod repository;
pub mod engine;

pub use engine::KpiEngine;
pub use repository::{KpiRepository, PostgresKpiRepository};
