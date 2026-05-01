//! Cash Flow Forecasting Module
//!
//! Oracle Fusion: Treasury > Cash Forecasting

mod engine;
pub use engine::CashFlowForecastEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecast { pub id: Uuid, pub organization_id: Uuid, pub forecast_number: String, pub name: String, pub description: Option<String>, pub status: String, pub forecast_horizon: String, pub periods_out: i32, pub start_date: chrono::NaiveDate, pub end_date: chrono::NaiveDate, pub base_currency_code: String, pub total_inflows: String, pub total_outflows: String, pub net_cash_flow: String, pub opening_balance: String, pub closing_balance: String, pub metadata: serde_json::Value, pub created_by: Option<Uuid>, pub approved_by: Option<Uuid>, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashScenario { pub id: Uuid, pub organization_id: Uuid, pub forecast_id: Uuid, pub scenario_number: String, pub name: String, pub description: Option<String>, pub scenario_type: String, pub adjustment_factor: String, pub total_inflows: String, pub total_outflows: String, pub net_cash_flow: String, pub opening_balance: String, pub closing_balance: String, pub metadata: serde_json::Value, pub created_by: Option<Uuid>, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashEntry { pub id: Uuid, pub organization_id: Uuid, pub forecast_id: Uuid, pub scenario_id: Option<Uuid>, pub period_name: String, pub period_start_date: chrono::NaiveDate, pub period_end_date: chrono::NaiveDate, pub source_category: String, pub flow_direction: String, pub amount: String, pub probability: String, pub weighted_amount: String, pub is_manual: bool, pub description: Option<String>, pub metadata: serde_json::Value, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastDashboard { pub total_forecasts: i32, pub active_forecasts: i32, pub total_projected_inflows: String, pub total_projected_outflows: String, pub net_projected_cash_flow: String, pub surplus_deficit: String }

#[async_trait]
pub trait CashFlowForecastRepository: Send + Sync {
    async fn create_forecast(&self, org_id: Uuid, fn_: &str, name: &str, desc: Option<&str>, horizon: &str, periods: i32, start: chrono::NaiveDate, end: chrono::NaiveDate, curr: &str, balance: &str, cb: Option<Uuid>) -> AtlasResult<CashForecast>;
    async fn get_forecast(&self, org_id: Uuid, fn_: &str) -> AtlasResult<Option<CashForecast>>;
    async fn get_forecast_by_id(&self, id: Uuid) -> AtlasResult<Option<CashForecast>>;
    async fn list_forecasts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CashForecast>>;
    async fn update_forecast_status(&self, id: Uuid, status: &str, ab: Option<Uuid>) -> AtlasResult<CashForecast>;
    async fn update_forecast_totals(&self, id: Uuid, inf: &str, out: &str, net: &str, closing: &str) -> AtlasResult<()>;
    async fn create_scenario(&self, org_id: Uuid, fid: Uuid, sn: &str, name: &str, desc: Option<&str>, st: &str, af: &str) -> AtlasResult<CashScenario>;
    async fn get_scenario(&self, id: Uuid) -> AtlasResult<Option<CashScenario>>;
    async fn list_scenarios(&self, fid: Uuid) -> AtlasResult<Vec<CashScenario>>;
    async fn update_scenario_totals(&self, id: Uuid, inf: &str, out: &str, net: &str, ob: &str, cb: &str) -> AtlasResult<CashScenario>;
    async fn create_entry(&self, org_id: Uuid, fid: Uuid, sid: Option<Uuid>, pn: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, cat: &str, dir: &str, amt: &str, prob: &str, man: bool, desc: Option<&str>) -> AtlasResult<CashEntry>;
    async fn list_entries(&self, fid: Uuid) -> AtlasResult<Vec<CashEntry>>;
    async fn delete_entry(&self, id: Uuid) -> AtlasResult<()>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashForecastDashboard>;
}

pub struct PostgresCashFlowForecastRepository { pool: PgPool }
impl PostgresCashFlowForecastRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl CashFlowForecastRepository for PostgresCashFlowForecastRepository {
    async fn create_forecast(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: i32, _: chrono::NaiveDate, _: chrono::NaiveDate, _: &str, _: &str, _: Option<Uuid>) -> AtlasResult<CashForecast> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_forecast(&self, _: Uuid, _: &str) -> AtlasResult<Option<CashForecast>> { Ok(None) }
    async fn get_forecast_by_id(&self, _: Uuid) -> AtlasResult<Option<CashForecast>> { Ok(None) }
    async fn list_forecasts(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<CashForecast>> { Ok(vec![]) }
    async fn update_forecast_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<CashForecast> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_forecast_totals(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_scenario(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: &str) -> AtlasResult<CashScenario> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_scenario(&self, _: Uuid) -> AtlasResult<Option<CashScenario>> { Ok(None) }
    async fn list_scenarios(&self, _: Uuid) -> AtlasResult<Vec<CashScenario>> { Ok(vec![]) }
    async fn update_scenario_totals(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: &str) -> AtlasResult<CashScenario> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_entry(&self, _: Uuid, _: Uuid, _: Option<Uuid>, _: &str, _: chrono::NaiveDate, _: chrono::NaiveDate, _: &str, _: &str, _: &str, _: &str, _: bool, _: Option<&str>) -> AtlasResult<CashEntry> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_entries(&self, _: Uuid) -> AtlasResult<Vec<CashEntry>> { Ok(vec![]) }
    async fn delete_entry(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CashForecastDashboard> { Ok(CashForecastDashboard { total_forecasts: 0, active_forecasts: 0, total_projected_inflows: "0".into(), total_projected_outflows: "0".into(), net_projected_cash_flow: "0".into(), surplus_deficit: "0".into() }) }
}
