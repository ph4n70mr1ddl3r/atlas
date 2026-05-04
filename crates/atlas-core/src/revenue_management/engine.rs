//! Revenue Management Engine (ASC 606 / IFRS 15)
//!
//! Manages revenue contracts, performance obligations, standalone selling
//! prices, and revenue recognition per ASC 606/IFRS 15.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Revenue Management

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_CONTRACT_STATUSES: &[&str] = &["draft", "active", "complete", "cancelled"];
const VALID_OBLIGATION_TYPES: &[&str] = &["goods", "services", "license", "bundle"];
const VALID_SATISFACTION_METHODS: &[&str] = &["point_in_time", "over_time"];
const VALID_RECOGNITION_PATTERNS: &[&str] = &["straight_line", "percentage_of_completion", "output_method", "input_method"];
const VALID_SSP_METHODS: &[&str] = &["observed", "adjusted_market", "expected_cost_plus_margin", "residual"];
#[allow(dead_code)]
const VALID_EVENT_TYPES: &[&str] = &["satisfaction", "partial_satisfaction", "adjustment", "reversal"];

pub struct RevenueManagementEngine {
    repository: Arc<dyn RevenueManagementRepository>,
}

impl RevenueManagementEngine {
    pub fn new(repository: Arc<dyn RevenueManagementRepository>) -> Self { Self { repository } }

    pub async fn create_contract(&self, org_id: Uuid, contract_number: &str, customer_id: Uuid, customer_name: &str, description: Option<&str>, transaction_price: &str, currency_code: &str, contract_start_date: chrono::NaiveDate, contract_end_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<RevMgmtContract> {
        if contract_number.is_empty() { return Err(AtlasError::ValidationFailed("Contract number is required".into())); }
        if customer_name.is_empty() { return Err(AtlasError::ValidationFailed("Customer name is required".into())); }
        let price: f64 = transaction_price.parse().unwrap_or(-1.0);
        if price < 0.0 { return Err(AtlasError::ValidationFailed("Transaction price must be non-negative".into())); }
        if currency_code.is_empty() { return Err(AtlasError::ValidationFailed("Currency code is required".into())); }
        if let Some(end) = contract_end_date { if end < contract_start_date { return Err(AtlasError::ValidationFailed("Contract end date must be after start date".into())); } }
        if self.repository.get_contract(org_id, contract_number).await?.is_some() { return Err(AtlasError::Conflict(format!("Contract number '{}' already exists", contract_number))); }
        info!("Creating revenue contract {} for org {}", contract_number, org_id);
        self.repository.create_contract(org_id, contract_number, customer_id, customer_name, description, transaction_price, currency_code, contract_start_date, contract_end_date, created_by).await
    }

    pub async fn get_contract(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<RevMgmtContract>> { self.repository.get_contract(org_id, contract_number).await }
    pub async fn get_contract_by_id(&self, id: Uuid) -> AtlasResult<Option<RevMgmtContract>> { self.repository.get_contract_by_id(id).await }

    pub async fn list_contracts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RevMgmtContract>> {
        if let Some(s) = status { if !VALID_CONTRACT_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'. Must be one of: {}", s, VALID_CONTRACT_STATUSES.join(", ")))); } }
        self.repository.list_contracts(org_id, status).await
    }

    pub async fn activate_contract(&self, contract_id: Uuid) -> AtlasResult<RevMgmtContract> {
        let c = self.repository.get_contract_by_id(contract_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;
        if c.status != "draft" { return Err(AtlasError::WorkflowError(format!("Cannot activate contract in '{}' status", c.status))); }
        info!("Activating revenue contract {}", c.contract_number);
        self.repository.update_contract_status(contract_id, "active").await
    }

    pub async fn complete_contract(&self, contract_id: Uuid) -> AtlasResult<RevMgmtContract> {
        let c = self.repository.get_contract_by_id(contract_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;
        if c.status != "active" { return Err(AtlasError::WorkflowError(format!("Cannot complete contract in '{}' status", c.status))); }
        let obs = self.repository.list_obligations_by_contract(contract_id).await?;
        if !obs.iter().all(|o| o.satisfaction_status == "satisfied") { return Err(AtlasError::WorkflowError("Not all obligations satisfied".into())); }
        info!("Completing revenue contract {}", c.contract_number);
        self.repository.update_contract_status(contract_id, "complete").await
    }

    pub async fn cancel_contract(&self, contract_id: Uuid) -> AtlasResult<RevMgmtContract> {
        let c = self.repository.get_contract_by_id(contract_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;
        if c.status != "draft" && c.status != "active" { return Err(AtlasError::WorkflowError(format!("Cannot cancel contract in '{}' status", c.status))); }
        self.repository.update_contract_status(contract_id, "cancelled").await
    }

    pub async fn create_obligation(&self, org_id: Uuid, contract_id: Uuid, obligation_number: &str, description: &str, obligation_type: &str, satisfaction_method: &str, recognition_pattern: &str, standalone_selling_price: &str, recognition_start_date: Option<chrono::NaiveDate>, recognition_end_date: Option<chrono::NaiveDate>) -> AtlasResult<RevMgmtObligation> {
        let contract = self.repository.get_contract_by_id(contract_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;
        if contract.status != "draft" && contract.status != "active" { return Err(AtlasError::WorkflowError("Can only add obligations to draft/active contracts".into())); }
        if description.is_empty() { return Err(AtlasError::ValidationFailed("Description is required".into())); }
        if !VALID_OBLIGATION_TYPES.contains(&obligation_type) { return Err(AtlasError::ValidationFailed(format!("Invalid obligation_type '{}'", obligation_type))); }
        if !VALID_SATISFACTION_METHODS.contains(&satisfaction_method) { return Err(AtlasError::ValidationFailed(format!("Invalid satisfaction_method '{}'", satisfaction_method))); }
        if !VALID_RECOGNITION_PATTERNS.contains(&recognition_pattern) { return Err(AtlasError::ValidationFailed(format!("Invalid recognition_pattern '{}'", recognition_pattern))); }
        let ssp: f64 = standalone_selling_price.parse().unwrap_or(-1.0);
        if ssp < 0.0 { return Err(AtlasError::ValidationFailed("SSP must be non-negative".into())); }
        self.repository.create_obligation(org_id, contract_id, Some(&contract.contract_number), obligation_number, description, obligation_type, satisfaction_method, recognition_pattern, standalone_selling_price, standalone_selling_price, recognition_start_date, recognition_end_date).await
    }

    pub async fn list_obligations(&self, contract_id: Uuid) -> AtlasResult<Vec<RevMgmtObligation>> { self.repository.list_obligations_by_contract(contract_id).await }

    pub async fn allocate_transaction_price(&self, contract_id: Uuid) -> AtlasResult<Vec<RevMgmtObligation>> {
        let contract = self.repository.get_contract_by_id(contract_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;
        if contract.status != "active" { return Err(AtlasError::WorkflowError("Can only allocate for active contracts".into())); }
        let obs = self.repository.list_obligations_by_contract(contract_id).await?;
        if obs.is_empty() { return Err(AtlasError::ValidationFailed("No obligations to allocate to".into())); }
        let tp: f64 = contract.transaction_price.parse().unwrap_or(0.0);
        let total_ssp: f64 = obs.iter().map(|o| o.standalone_selling_price.parse::<f64>().unwrap_or(0.0)).sum();
        if total_ssp <= 0.0 { return Err(AtlasError::ValidationFailed("Total SSP must be > 0".into())); }
        let mut allocated_total = 0.0_f64;
        let mut updated = Vec::new();
        for (i, o) in obs.iter().enumerate() {
            let ssp: f64 = o.standalone_selling_price.parse().unwrap_or(0.0);
            let alloc = if i == obs.len() - 1 { (tp - allocated_total).max(0.0) } else { (tp * ssp / total_ssp * 100.0).round() / 100.0 };
            allocated_total += alloc;
            let unrec = alloc - o.recognized_amount.parse::<f64>().unwrap_or(0.0);
            let pct = if alloc > 0.0 { o.recognized_amount.parse::<f64>().unwrap_or(0.0) / alloc * 100.0 } else { 0.0 };
            let u = self.repository.update_obligation_status(o.id, &o.satisfaction_status, &o.recognized_amount, &unrec.to_string(), &format!("{:.2}", pct)).await?;
            updated.push(u);
        }
        let tot_rec: f64 = updated.iter().map(|o| o.recognized_amount.parse::<f64>().unwrap_or(0.0)).sum();
        let sat = updated.iter().filter(|o| o.satisfaction_status == "satisfied").count() as i32;
        self.repository.update_contract_totals(contract_id, &allocated_total.to_string(), &tot_rec.to_string(), &(allocated_total - tot_rec).to_string(), updated.len() as i32, sat).await?;
        Ok(updated)
    }

    pub async fn create_ssp(&self, org_id: Uuid, item_code: &str, item_name: &str, estimation_method: &str, price: &str, currency_code: &str, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<RevMgmtSSP> {
        if item_code.is_empty() || item_name.is_empty() { return Err(AtlasError::ValidationFailed("Item code and name required".into())); }
        if !VALID_SSP_METHODS.contains(&estimation_method) { return Err(AtlasError::ValidationFailed(format!("Invalid estimation_method '{}'", estimation_method))); }
        let p: f64 = price.parse().unwrap_or(-1.0);
        if p < 0.0 { return Err(AtlasError::ValidationFailed("Price must be non-negative".into())); }
        if let Some(to) = effective_to { if to < effective_from { return Err(AtlasError::ValidationFailed("Effective to must be after from".into())); } }
        self.repository.create_ssp(org_id, item_code, item_name, estimation_method, price, currency_code, effective_from, effective_to, created_by).await
    }

    pub async fn get_ssp(&self, org_id: Uuid, item_code: &str, on_date: chrono::NaiveDate) -> AtlasResult<Option<RevMgmtSSP>> { self.repository.get_ssp(org_id, item_code, on_date).await }
    pub async fn list_ssps(&self, org_id: Uuid) -> AtlasResult<Vec<RevMgmtSSP>> { self.repository.list_ssps(org_id).await }

    pub async fn satisfy_obligation(&self, obligation_id: Uuid, amount: &str, recognition_date: chrono::NaiveDate, gl_account_code: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<RevMgmtRecognitionEvent> {
        let o = self.repository.get_obligation(obligation_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Obligation {} not found", obligation_id)))?;
        let rec_amt: f64 = amount.parse().unwrap_or(-1.0);
        if rec_amt <= 0.0 { return Err(AtlasError::ValidationFailed("Amount must be positive".into())); }
        let cur_rec: f64 = o.recognized_amount.parse().unwrap_or(0.0);
        let alloc: f64 = o.allocated_amount.parse().unwrap_or(0.0);
        let new_total = cur_rec + rec_amt;
        if new_total > alloc + 0.01 { return Err(AtlasError::ValidationFailed(format!("Amount {} exceeds allocated {}", new_total, alloc))); }
        let full = (new_total - alloc).abs() < 0.01;
        let etype = if full { "satisfaction" } else { "partial_satisfaction" };
        let new_status = if full { "satisfied" } else { "in_progress" };
        let new_unrec = (alloc - new_total).max(0.0);
        let new_pct = if alloc > 0.0 { new_total / alloc * 100.0 } else { 0.0 };
        let evnum = format!("RRE-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
        let event = self.repository.create_recognition_event(o.organization_id, o.contract_id, obligation_id, &evnum, &format!("{} of obligation {}", etype, o.obligation_number), etype, amount, recognition_date, gl_account_code, created_by).await?;
        self.repository.update_obligation_status(obligation_id, new_status, &new_total.to_string(), &new_unrec.to_string(), &format!("{:.2}", new_pct)).await?;
        let all_obs = self.repository.list_obligations_by_contract(o.contract_id).await?;
        let tot_rec: f64 = all_obs.iter().map(|ob| ob.recognized_amount.parse::<f64>().unwrap_or(0.0)).sum();
        let tot_alloc: f64 = all_obs.iter().map(|ob| ob.allocated_amount.parse::<f64>().unwrap_or(0.0)).sum();
        let sat = all_obs.iter().filter(|ob| ob.satisfaction_status == "satisfied").count() as i32;
        self.repository.update_contract_totals(o.contract_id, &tot_alloc.to_string(), &tot_rec.to_string(), &(tot_alloc - tot_rec).to_string(), all_obs.len() as i32, sat).await?;
        Ok(event)
    }

    pub async fn list_recognition_events(&self, contract_id: Uuid) -> AtlasResult<Vec<RevMgmtRecognitionEvent>> { self.repository.list_events_by_contract(contract_id).await }
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RevMgmtDashboard> { self.repository.get_dashboard(org_id).await }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        contracts: std::sync::Mutex<Vec<RevMgmtContract>>,
    }
    impl MockRepo { fn new() -> Self { MockRepo { contracts: std::sync::Mutex::new(vec![]) } } }
    #[async_trait::async_trait]
    impl RevenueManagementRepository for MockRepo {
        async fn create_contract(&self, org_id: Uuid, cn: &str, cid: Uuid, cname: &str, desc: Option<&str>, tp: &str, cc: &str, csd: chrono::NaiveDate, ced: Option<chrono::NaiveDate>, cb: Option<Uuid>) -> AtlasResult<RevMgmtContract> {
            let c = RevMgmtContract { id: Uuid::new_v4(), organization_id: org_id, contract_number: cn.into(), customer_id: cid, customer_name: cname.into(), description: desc.map(Into::into), status: "draft".into(), transaction_price: tp.into(), total_allocated: "0".into(), total_recognized: "0".into(), total_unrecognized: tp.into(), currency_code: cc.into(), contract_start_date: csd, contract_end_date: ced, performance_obligation_count: 0, satisfied_obligation_count: 0, metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.contracts.lock().unwrap().push(c.clone());
            Ok(c)
        }
        async fn get_contract(&self, org_id: Uuid, cn: &str) -> AtlasResult<Option<RevMgmtContract>> { Ok(self.contracts.lock().unwrap().iter().find(|c| c.organization_id == org_id && c.contract_number == cn).cloned()) }
        async fn get_contract_by_id(&self, id: Uuid) -> AtlasResult<Option<RevMgmtContract>> { Ok(self.contracts.lock().unwrap().iter().find(|c| c.id == id).cloned()) }
        async fn list_contracts(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<RevMgmtContract>> { Ok(vec![]) }
        async fn update_contract_status(&self, id: Uuid, status: &str) -> AtlasResult<RevMgmtContract> { Err(AtlasError::EntityNotFound("Mock".into())) }
        async fn update_contract_totals(&self, _: Uuid, _: &str, _: &str, _: &str, _: i32, _: i32) -> AtlasResult<()> { Ok(()) }
        async fn create_obligation(&self, org_id: Uuid, contract_id: Uuid, cn: Option<&str>, on: &str, desc: &str, ot: &str, sm: &str, rp: &str, ssp: &str, aa: &str, rsd: Option<chrono::NaiveDate>, red: Option<chrono::NaiveDate>) -> AtlasResult<RevMgmtObligation> {
            Ok(RevMgmtObligation { id: Uuid::new_v4(), organization_id: org_id, contract_id, contract_number: cn.map(Into::into), obligation_number: on.into(), description: desc.into(), obligation_type: ot.into(), satisfaction_status: "not_started".into(), satisfaction_method: sm.into(), recognition_pattern: rp.into(), standalone_selling_price: ssp.into(), allocated_amount: aa.into(), recognized_amount: "0".into(), unrecognized_amount: aa.into(), recognition_start_date: rsd, recognition_end_date: red, percent_complete: "0".into(), metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn get_obligation(&self, _: Uuid) -> AtlasResult<Option<RevMgmtObligation>> { Ok(None) }
        async fn list_obligations_by_contract(&self, _: Uuid) -> AtlasResult<Vec<RevMgmtObligation>> { Ok(vec![]) }
        async fn update_obligation_status(&self, id: Uuid, ss: &str, ra: &str, ua: &str, pc: &str) -> AtlasResult<RevMgmtObligation> { Err(AtlasError::EntityNotFound("Mock".into())) }
        async fn create_ssp(&self, org_id: Uuid, ic: &str, in_: &str, em: &str, p: &str, cc: &str, ef: chrono::NaiveDate, et: Option<chrono::NaiveDate>, cb: Option<Uuid>) -> AtlasResult<RevMgmtSSP> {
            Ok(RevMgmtSSP { id: Uuid::new_v4(), organization_id: org_id, item_code: ic.into(), item_name: in_.into(), estimation_method: em.into(), price: p.into(), currency_code: cc.into(), effective_from: ef, effective_to: et, is_active: true, metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn get_ssp(&self, _: Uuid, _: &str, _: chrono::NaiveDate) -> AtlasResult<Option<RevMgmtSSP>> { Ok(None) }
        async fn list_ssps(&self, _: Uuid) -> AtlasResult<Vec<RevMgmtSSP>> { Ok(vec![]) }
        async fn create_recognition_event(&self, org_id: Uuid, contract_id: Uuid, obl_id: Uuid, en: &str, desc: &str, et: &str, amt: &str, rd: chrono::NaiveDate, glac: Option<&str>, cb: Option<Uuid>) -> AtlasResult<RevMgmtRecognitionEvent> {
            Ok(RevMgmtRecognitionEvent { id: Uuid::new_v4(), organization_id: org_id, contract_id, obligation_id: obl_id, event_number: en.into(), description: desc.into(), event_type: et.into(), amount: amt.into(), recognition_date: rd, gl_account_code: glac.map(Into::into), is_posted: false, posted_at: None, metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn list_events_by_contract(&self, _: Uuid) -> AtlasResult<Vec<RevMgmtRecognitionEvent>> { Ok(vec![]) }
        async fn list_events_by_obligation(&self, _: Uuid) -> AtlasResult<Vec<RevMgmtRecognitionEvent>> { Ok(vec![]) }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<RevMgmtDashboard> { Ok(RevMgmtDashboard { total_contracts: 0, active_contracts: 0, total_performance_obligations: 0, satisfied_obligations: 0, total_transaction_price: "0".into(), total_allocated: "0".into(), total_recognized: "0".into(), total_unrecognized: "0".into(), unrecognized_by_period: serde_json::json!([]) }) }
    }

    fn eng() -> RevenueManagementEngine { RevenueManagementEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_CONTRACT_STATUSES.len(), 4);
        assert_eq!(VALID_OBLIGATION_TYPES.len(), 4);
        assert_eq!(VALID_SATISFACTION_METHODS.len(), 2);
        assert_eq!(VALID_RECOGNITION_PATTERNS.len(), 4);
        assert_eq!(VALID_SSP_METHODS.len(), 4);
        assert_eq!(VALID_EVENT_TYPES.len(), 4);
    }

    #[tokio::test]
    async fn test_create_contract_valid() {
        let c = eng().create_contract(Uuid::new_v4(), "RC-001", Uuid::new_v4(), "Cust", None, "10000.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        assert_eq!(c.contract_number, "RC-001");
        assert_eq!(c.status, "draft");
    }

    #[tokio::test]
    async fn test_create_contract_empty_number() {
        let r = eng().create_contract(Uuid::new_v4(), "", Uuid::new_v4(), "Cust", None, "100.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_contract_negative_price() {
        let r = eng().create_contract(Uuid::new_v4(), "RC-1", Uuid::new_v4(), "Cust", None, "-100.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_contract_end_before_start() {
        let r = eng().create_contract(Uuid::new_v4(), "RC-1", Uuid::new_v4(), "Cust", None, "100.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,6,1).unwrap(), Some(chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap()), None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_contract_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_contract(org, "RC-DUP", Uuid::new_v4(), "C1", None, "100.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        let r = e.create_contract(org, "RC-DUP", Uuid::new_v4(), "C2", None, "200.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_list_contracts_invalid_status() {
        let r = eng().list_contracts(Uuid::new_v4(), Some("bad")).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_contracts_valid() {
        let r = eng().list_contracts(Uuid::new_v4(), Some("draft")).await;
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_create_obligation_invalid_type() {
        let org = Uuid::new_v4();
        let e = eng();
        let c = e.create_contract(org, "RC-O1", Uuid::new_v4(), "C", None, "5000.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let r = e.create_obligation(org, c.id, "PO-1", "Desc", "bad", "point_in_time", "straight_line", "3000.00", None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_obligation_invalid_method() {
        let org = Uuid::new_v4();
        let e = eng();
        let c = e.create_contract(org, "RC-O2", Uuid::new_v4(), "C", None, "5000.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let r = e.create_obligation(org, c.id, "PO-1", "Desc", "goods", "bad", "straight_line", "3000.00", None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_obligation_valid() {
        let org = Uuid::new_v4();
        let e = eng();
        let c = e.create_contract(org, "RC-O3", Uuid::new_v4(), "C", None, "10000.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        let o = e.create_obligation(org, c.id, "PO-1", "License", "license", "point_in_time", "straight_line", "6000.00", None, None).await.unwrap();
        assert_eq!(o.obligation_number, "PO-1");
        assert_eq!(o.satisfaction_status, "not_started");
    }

    #[tokio::test]
    async fn test_create_ssp_valid() {
        let s = eng().create_ssp(Uuid::new_v4(), "ITEM-1", "Software", "observed", "500.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await.unwrap();
        assert_eq!(s.item_code, "ITEM-1");
    }

    #[tokio::test]
    async fn test_create_ssp_empty_code() {
        let r = eng().create_ssp(Uuid::new_v4(), "", "Software", "observed", "500.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_ssp_invalid_method() {
        let r = eng().create_ssp(Uuid::new_v4(), "I1", "S", "bad", "500.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_ssp_negative_price() {
        let r = eng().create_ssp(Uuid::new_v4(), "I1", "S", "observed", "-1.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_ssp_effective_to_before_from() {
        let r = eng().create_ssp(Uuid::new_v4(), "I1", "S", "observed", "500.00", "USD", chrono::NaiveDate::from_ymd_opt(2026,6,1).unwrap(), Some(chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap()), None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_satisfy_not_found() {
        let r = eng().satisfy_obligation(Uuid::new_v4(), "100.00", chrono::NaiveDate::from_ymd_opt(2026,3,1).unwrap(), None, None).await;
        assert!(matches!(r, Err(AtlasError::EntityNotFound(_))));
    }
}
