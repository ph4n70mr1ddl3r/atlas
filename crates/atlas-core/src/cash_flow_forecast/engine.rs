//! Cash Flow Forecasting Engine
//! Oracle Fusion: Treasury > Cash Forecasting

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_STATUSES: &[&str] = &["draft", "active", "approved", "superseded"];
const VALID_HORIZONS: &[&str] = &["daily", "weekly", "monthly", "quarterly"];
const VALID_SCENARIO_TYPES: &[&str] = &["best_case", "worst_case", "most_likely", "custom"];
const VALID_CATEGORIES: &[&str] = &["accounts_receivable", "accounts_payable", "payroll", "debt_service", "capital_expenditure", "operating", "investing", "financing", "other"];
const VALID_DIRECTIONS: &[&str] = &["inflow", "outflow"];

pub struct CashFlowForecastEngine { repository: Arc<dyn CashFlowForecastRepository> }
impl CashFlowForecastEngine {
    pub fn new(r: Arc<dyn CashFlowForecastRepository>) -> Self { Self { repository: r } }

    pub async fn create_forecast(&self, org_id: Uuid, forecast_number: &str, name: &str, description: Option<&str>, forecast_horizon: &str, periods_out: i32, start_date: chrono::NaiveDate, end_date: chrono::NaiveDate, base_currency_code: &str, opening_balance: &str, created_by: Option<Uuid>) -> AtlasResult<CashForecast> {
        if forecast_number.is_empty() || name.is_empty() { return Err(AtlasError::ValidationFailed("Forecast number and name required".into())); }
        if !VALID_HORIZONS.contains(&forecast_horizon) { return Err(AtlasError::ValidationFailed(format!("Invalid horizon '{}'", forecast_horizon))); }
        if periods_out < 1 || periods_out > 120 { return Err(AtlasError::ValidationFailed("Periods out must be 1-120".into())); }
        if end_date <= start_date { return Err(AtlasError::ValidationFailed("End date must be after start".into())); }
        let bal: f64 = opening_balance.parse().map_err(|_| AtlasError::ValidationFailed("Invalid balance".into()))?;
        if bal < 0.0 { return Err(AtlasError::ValidationFailed("Balance must be non-negative".into())); }
        if self.repository.get_forecast(org_id, forecast_number).await?.is_some() { return Err(AtlasError::Conflict(format!("Forecast '{}' already exists", forecast_number))); }
        info!("Creating forecast {} for org {}", forecast_number, org_id);
        self.repository.create_forecast(org_id, forecast_number, name, description, forecast_horizon, periods_out, start_date, end_date, base_currency_code, opening_balance, created_by).await
    }
    pub async fn get_forecast(&self, org_id: Uuid, fn_: &str) -> AtlasResult<Option<CashForecast>> { self.repository.get_forecast(org_id, fn_).await }
    pub async fn get_forecast_by_id(&self, id: Uuid) -> AtlasResult<Option<CashForecast>> { self.repository.get_forecast_by_id(id).await }
    pub async fn list_forecasts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CashForecast>> {
        if let Some(s) = status { if !VALID_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s))); } }
        self.repository.list_forecasts(org_id, status).await
    }
    pub async fn activate_forecast(&self, fid: Uuid) -> AtlasResult<CashForecast> {
        let f = self.repository.get_forecast_by_id(fid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Forecast {} not found", fid)))?;
        if f.status != "draft" { return Err(AtlasError::WorkflowError(format!("Cannot activate in '{}' status", f.status))); }
        self.repository.update_forecast_status(fid, "active", None).await
    }
    pub async fn approve_forecast(&self, fid: Uuid, ab: Uuid) -> AtlasResult<CashForecast> {
        let f = self.repository.get_forecast_by_id(fid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Forecast {} not found", fid)))?;
        if f.status != "active" { return Err(AtlasError::WorkflowError("Must be active to approve".into())); }
        self.repository.update_forecast_status(fid, "approved", Some(ab)).await
    }
    pub async fn recalculate(&self, fid: Uuid) -> AtlasResult<CashForecast> {
        let f = self.repository.get_forecast_by_id(fid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Forecast {} not found", fid)))?;
        let entries = self.repository.list_entries(fid).await?;
        let inf: f64 = entries.iter().filter(|e| e.flow_direction == "inflow").map(|e| e.weighted_amount.parse::<f64>().unwrap_or(0.0)).sum();
        let out: f64 = entries.iter().filter(|e| e.flow_direction == "outflow").map(|e| e.weighted_amount.parse::<f64>().unwrap_or(0.0)).sum();
        let net = inf - out;
        let open: f64 = f.opening_balance.parse().unwrap_or(0.0);
        self.repository.update_forecast_totals(fid, &inf.to_string(), &out.to_string(), &net.to_string(), &(open + net).to_string()).await?;
        self.repository.get_forecast_by_id(fid).await?.ok_or(AtlasError::EntityNotFound("gone".into()))
    }
    pub async fn create_scenario(&self, org_id: Uuid, fid: Uuid, sn: &str, name: &str, desc: Option<&str>, st: &str, af: &str) -> AtlasResult<CashScenario> {
        self.repository.get_forecast_by_id(fid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Forecast {} not found", fid)))?;
        if name.is_empty() { return Err(AtlasError::ValidationFailed("Name required".into())); }
        if !VALID_SCENARIO_TYPES.contains(&st) { return Err(AtlasError::ValidationFailed(format!("Invalid scenario_type '{}'", st))); }
        let factor: f64 = af.parse().unwrap_or(0.0);
        if factor < 0.0 || factor > 10.0 { return Err(AtlasError::ValidationFailed("Factor must be 0.0-10.0".into())); }
        self.repository.create_scenario(org_id, fid, sn, name, desc, st, af).await
    }
    pub async fn list_scenarios(&self, fid: Uuid) -> AtlasResult<Vec<CashScenario>> { self.repository.list_scenarios(fid).await }
    pub async fn create_entry(&self, org_id: Uuid, fid: Uuid, sid: Option<Uuid>, pn: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, cat: &str, dir: &str, amt: &str, prob: &str, man: bool, desc: Option<&str>) -> AtlasResult<CashEntry> {
        if pn.is_empty() { return Err(AtlasError::ValidationFailed("Period name required".into())); }
        if !VALID_CATEGORIES.contains(&cat) { return Err(AtlasError::ValidationFailed(format!("Invalid category '{}'", cat))); }
        if !VALID_DIRECTIONS.contains(&dir) { return Err(AtlasError::ValidationFailed(format!("Invalid direction '{}'", dir))); }
        let a: f64 = amt.parse().map_err(|_| AtlasError::ValidationFailed("Invalid amount".into()))?;
        if a < 0.0 { return Err(AtlasError::ValidationFailed("Amount must be non-negative".into())); }
        let p: f64 = prob.parse().map_err(|_| AtlasError::ValidationFailed("Invalid probability".into()))?;
        if p < 0.0 || p > 1.0 { return Err(AtlasError::ValidationFailed("Probability must be 0.0-1.0".into())); }
        if pe < ps { return Err(AtlasError::ValidationFailed("End must be after start".into())); }
        self.repository.create_entry(org_id, fid, sid, pn, ps, pe, cat, dir, amt, prob, man, desc).await
    }
    pub async fn list_entries(&self, fid: Uuid) -> AtlasResult<Vec<CashEntry>> { self.repository.list_entries(fid).await }
    pub async fn delete_entry(&self, id: Uuid) -> AtlasResult<()> { self.repository.delete_entry(id).await }
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashForecastDashboard> { self.repository.get_dashboard(org_id).await }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock { forecasts: std::sync::Mutex<Vec<CashForecast>> }
    impl Mock { fn new() -> Self { Mock { forecasts: std::sync::Mutex::new(vec![]) } } }
    #[async_trait]
    impl CashFlowForecastRepository for Mock {
        async fn create_forecast(&self, org_id: Uuid, fn_: &str, name: &str, desc: Option<&str>, horizon: &str, periods: i32, start: chrono::NaiveDate, end: chrono::NaiveDate, curr: &str, bal: &str, cb: Option<Uuid>) -> AtlasResult<CashForecast> {
            let f = CashForecast { id: Uuid::new_v4(), organization_id: org_id, forecast_number: fn_.into(), name: name.into(), description: desc.map(Into::into), status: "draft".into(), forecast_horizon: horizon.into(), periods_out: periods, start_date: start, end_date: end, base_currency_code: curr.into(), total_inflows: "0".into(), total_outflows: "0".into(), net_cash_flow: "0".into(), opening_balance: bal.into(), closing_balance: bal.into(), metadata: serde_json::json!({}), created_by: cb, approved_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.forecasts.lock().unwrap().push(f.clone());
            Ok(f)
        }
        async fn get_forecast(&self, org_id: Uuid, fn_: &str) -> AtlasResult<Option<CashForecast>> { Ok(self.forecasts.lock().unwrap().iter().find(|f| f.organization_id == org_id && f.forecast_number == fn_).cloned()) }
        async fn get_forecast_by_id(&self, _: Uuid) -> AtlasResult<Option<CashForecast>> { Ok(None) }
        async fn list_forecasts(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<CashForecast>> { Ok(vec![]) }
        async fn update_forecast_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<CashForecast> { Err(AtlasError::EntityNotFound("Mock".into())) }
        async fn update_forecast_totals(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
        async fn create_scenario(&self, org_id: Uuid, fid: Uuid, sn: &str, name: &str, desc: Option<&str>, st: &str, af: &str) -> AtlasResult<CashScenario> {
            Ok(CashScenario { id: Uuid::new_v4(), organization_id: org_id, forecast_id: fid, scenario_number: sn.into(), name: name.into(), description: desc.map(Into::into), scenario_type: st.into(), adjustment_factor: af.into(), total_inflows: "0".into(), total_outflows: "0".into(), net_cash_flow: "0".into(), opening_balance: "0".into(), closing_balance: "0".into(), metadata: serde_json::json!({}), created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn get_scenario(&self, _: Uuid) -> AtlasResult<Option<CashScenario>> { Ok(None) }
        async fn list_scenarios(&self, _: Uuid) -> AtlasResult<Vec<CashScenario>> { Ok(vec![]) }
        async fn update_scenario_totals(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: &str) -> AtlasResult<CashScenario> { Err(AtlasError::EntityNotFound("Mock".into())) }
        async fn create_entry(&self, org_id: Uuid, fid: Uuid, sid: Option<Uuid>, pn: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, cat: &str, dir: &str, amt: &str, prob: &str, man: bool, desc: Option<&str>) -> AtlasResult<CashEntry> {
            let a: f64 = amt.parse().unwrap_or(0.0); let p: f64 = prob.parse().unwrap_or(1.0); let w = (a * p * 100.0).round() / 100.0;
            Ok(CashEntry { id: Uuid::new_v4(), organization_id: org_id, forecast_id: fid, scenario_id: sid, period_name: pn.into(), period_start_date: ps, period_end_date: pe, source_category: cat.into(), flow_direction: dir.into(), amount: amt.into(), probability: prob.into(), weighted_amount: w.to_string(), is_manual: man, description: desc.map(Into::into), metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn list_entries(&self, _: Uuid) -> AtlasResult<Vec<CashEntry>> { Ok(vec![]) }
        async fn delete_entry(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CashForecastDashboard> { Ok(CashForecastDashboard { total_forecasts: 0, active_forecasts: 0, total_projected_inflows: "0".into(), total_projected_outflows: "0".into(), net_projected_cash_flow: "0".into(), surplus_deficit: "0".into() }) }
    }
    fn eng() -> CashFlowForecastEngine { CashFlowForecastEngine::new(Arc::new(Mock::new())) }

    #[test]
    fn test_valid_constants() { assert_eq!(VALID_STATUSES.len(), 4); assert_eq!(VALID_HORIZONS.len(), 4); assert_eq!(VALID_SCENARIO_TYPES.len(), 4); assert_eq!(VALID_CATEGORIES.len(), 9); assert_eq!(VALID_DIRECTIONS.len(), 2); }

    #[tokio::test]
    async fn test_create_forecast_valid() { let f = eng().create_forecast(Uuid::new_v4(), "CF-001", "Q2", None, "monthly", 3, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "500000.00", None).await.unwrap(); assert_eq!(f.forecast_number, "CF-001"); assert_eq!(f.status, "draft"); }

    #[tokio::test]
    async fn test_create_forecast_empty_number() { assert!(eng().create_forecast(Uuid::new_v4(), "", "Q2", None, "monthly", 3, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "1000.00", None).await.is_err()); }

    #[tokio::test]
    async fn test_create_forecast_invalid_horizon() { assert!(eng().create_forecast(Uuid::new_v4(), "CF-1", "T", None, "yearly", 3, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "1000.00", None).await.is_err()); }

    #[tokio::test]
    async fn test_create_forecast_periods_zero() { assert!(eng().create_forecast(Uuid::new_v4(), "CF-1", "T", None, "monthly", 0, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "1000.00", None).await.is_err()); }

    #[tokio::test]
    async fn test_create_forecast_end_before_start() { assert!(eng().create_forecast(Uuid::new_v4(), "CF-1", "T", None, "monthly", 3, chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), "USD", "1000.00", None).await.is_err()); }

    #[tokio::test]
    async fn test_create_forecast_negative_balance() { assert!(eng().create_forecast(Uuid::new_v4(), "CF-1", "T", None, "monthly", 3, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "-1000.00", None).await.is_err()); }

    #[tokio::test]
    async fn test_create_forecast_duplicate() { let org = Uuid::new_v4(); let e = eng(); let _ = e.create_forecast(org, "CF-DUP", "T1", None, "monthly", 3, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "1000.00", None).await; assert!(e.create_forecast(org, "CF-DUP", "T2", None, "monthly", 3, chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), "USD", "2000.00", None).await.is_err()); }

    #[tokio::test]
    async fn test_create_entry_invalid_category() { assert!(eng().create_entry(Uuid::new_v4(), Uuid::new_v4(), None, "Apr", chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,4,30).unwrap(), "bad", "inflow", "50000.00", "0.8", true, None).await.is_err()); }

    #[tokio::test]
    async fn test_create_entry_invalid_direction() { assert!(eng().create_entry(Uuid::new_v4(), Uuid::new_v4(), None, "Apr", chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,4,30).unwrap(), "accounts_receivable", "sideways", "50000.00", "0.8", true, None).await.is_err()); }

    #[tokio::test]
    async fn test_create_entry_negative_amount() { assert!(eng().create_entry(Uuid::new_v4(), Uuid::new_v4(), None, "Apr", chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,4,30).unwrap(), "accounts_receivable", "inflow", "-500.00", "0.8", true, None).await.is_err()); }

    #[tokio::test]
    async fn test_create_entry_bad_probability() { assert!(eng().create_entry(Uuid::new_v4(), Uuid::new_v4(), None, "Apr", chrono::NaiveDate::from_ymd_opt(2026,4,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,4,30).unwrap(), "accounts_receivable", "inflow", "50000.00", "1.5", true, None).await.is_err()); }

    #[tokio::test]
    async fn test_create_scenario_invalid_type() { assert!(eng().create_scenario(Uuid::new_v4(), Uuid::new_v4(), "SC-1", "T", None, "bad", "1.0").await.is_err()); }

    #[tokio::test]
    async fn test_create_scenario_bad_factor() { assert!(eng().create_scenario(Uuid::new_v4(), Uuid::new_v4(), "SC-1", "T", None, "best_case", "15.0").await.is_err()); }

    #[tokio::test]
    async fn test_list_forecasts_invalid_status() { assert!(eng().list_forecasts(Uuid::new_v4(), Some("bad")).await.is_err()); }
}
