//! AR Aging Analysis Engine
//! Oracle Fusion: AR > Aging Reports

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_STATUSES: &[&str] = &["active", "inactive"];
const VALID_AGING_BASES: &[&str] = &["invoice_date", "due_date", "trx_date"];

pub struct ArAgingEngine { repository: Arc<dyn ArAgingRepository> }

impl ArAgingEngine {
    pub fn new(r: Arc<dyn ArAgingRepository>) -> Self { Self { repository: r } }

    pub async fn create_definition(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, aging_basis: &str, num_buckets: i32, created_by: Option<Uuid>) -> AtlasResult<ArAgingDefinition> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if !VALID_AGING_BASES.contains(&aging_basis) {
            return Err(AtlasError::ValidationFailed(format!("Invalid aging_basis '{}'", aging_basis)));
        }
        if !(1..=20).contains(&num_buckets) {
            return Err(AtlasError::ValidationFailed("Number of buckets must be 1-20".into()));
        }
        if self.repository.get_definition(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Definition '{}' already exists", code)));
        }
        info!("Creating AR aging definition {} for org {}", code, org_id);
        self.repository.create_definition(org_id, code, name, description, aging_basis, num_buckets, created_by).await
    }

    pub async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<ArAgingDefinition>> {
        self.repository.get_definition_by_id(id).await
    }

    pub async fn list_definitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ArAgingDefinition>> {
        if let Some(s) = status { if !VALID_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s))); } }
        self.repository.list_definitions(org_id, status).await
    }

    pub async fn delete_definition(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.get_definition_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", id)))?;
        self.repository.delete_definition(id).await
    }

    pub async fn create_bucket(&self, org_id: Uuid, def_id: Uuid, bucket_number: i32, name: &str, from_days: i32, to_days: Option<i32>, display_order: i32) -> AtlasResult<ArAgingBucket> {
        self.repository.get_definition_by_id(def_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", def_id)))?;
        if name.is_empty() { return Err(AtlasError::ValidationFailed("Bucket name is required".into())); }
        if bucket_number < 0 { return Err(AtlasError::ValidationFailed("Bucket number must be >= 0".into())); }
        if from_days < 0 { return Err(AtlasError::ValidationFailed("From days must be >= 0".into())); }
        if let Some(to) = to_days {
            if to < from_days { return Err(AtlasError::ValidationFailed("To days must be >= from days".into())); }
        }
        self.repository.create_bucket(org_id, def_id, bucket_number, name, from_days, to_days, display_order).await
    }

    pub async fn list_buckets(&self, def_id: Uuid) -> AtlasResult<Vec<ArAgingBucket>> {
        self.repository.list_buckets(def_id).await
    }

    /// Determine which bucket a given days_past_due value falls into
    pub fn determine_bucket<'a>(&self, days_past_due: i32, buckets: &'a [ArAgingBucket]) -> Option<&'a ArAgingBucket> {
        buckets.iter().find(|b| {
            days_past_due >= b.from_days && (b.to_days.is_none() || days_past_due <= b.to_days.unwrap())
        })
    }

    pub async fn create_snapshot(&self, org_id: Uuid, def_id: Uuid, as_of_date: chrono::NaiveDate, currency_code: &str, created_by: Option<Uuid>) -> AtlasResult<ArAgingSnapshot> {
        let def = self.repository.get_definition_by_id(def_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", def_id)))?;
        if def.status != "active" { return Err(AtlasError::WorkflowError("Definition must be active".into())); }
        if currency_code.len() != 3 { return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into())); }
        info!("Creating AR aging snapshot for definition {} as of {}", def.definition_code, as_of_date);
        self.repository.create_snapshot(org_id, def_id, as_of_date, currency_code, created_by).await
    }

    pub async fn get_snapshot(&self, id: Uuid) -> AtlasResult<Option<ArAgingSnapshot>> {
        self.repository.get_snapshot(id).await
    }

    pub async fn list_snapshots(&self, org_id: Uuid, def_id: Option<Uuid>) -> AtlasResult<Vec<ArAgingSnapshot>> {
        self.repository.list_snapshots(org_id, def_id).await
    }

    pub async fn add_snapshot_line(&self, org_id: Uuid, snapshot_id: Uuid, customer_id: Option<Uuid>, customer_number: &str, customer_name: Option<&str>, invoice_id: Option<Uuid>, invoice_number: &str, invoice_date: chrono::NaiveDate, due_date: chrono::NaiveDate, original_amount: &str, open_amount: &str, days_past_due: i32, bucket_number: i32, bucket_name: &str, currency_code: &str) -> AtlasResult<ArAgingSnapshotLine> {
        if customer_number.is_empty() || invoice_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Customer number and invoice number are required".into()));
        }
        let oa: f64 = open_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid open amount".into()))?;
        if oa < 0.0 { return Err(AtlasError::ValidationFailed("Open amount must be non-negative".into())); }
        self.repository.create_snapshot_line(org_id, snapshot_id, customer_id, customer_number, customer_name, invoice_id, invoice_number, invoice_date, due_date, original_amount, open_amount, days_past_due, bucket_number, bucket_name, currency_code).await
    }

    pub async fn list_snapshot_lines(&self, snapshot_id: Uuid) -> AtlasResult<Vec<ArAgingSnapshotLine>> {
        self.repository.list_snapshot_lines(snapshot_id).await
    }

    pub async fn get_aging_summary(&self, snapshot_id: Uuid) -> AtlasResult<Vec<ArAgingSummary>> {
        self.repository.get_aging_summary(snapshot_id).await
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ArAgingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        defs: std::sync::Mutex<Vec<ArAgingDefinition>>,
        buckets: std::sync::Mutex<Vec<ArAgingBucket>>,
        snapshots: std::sync::Mutex<Vec<ArAgingSnapshot>>,
        lines: std::sync::Mutex<Vec<ArAgingSnapshotLine>>,
    }
    impl MockRepo { fn new() -> Self { Self { defs: std::sync::Mutex::new(vec![]), buckets: std::sync::Mutex::new(vec![]), snapshots: std::sync::Mutex::new(vec![]), lines: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl ArAgingRepository for MockRepo {
        async fn create_definition(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, basis: &str, nb: i32, cb: Option<Uuid>) -> AtlasResult<ArAgingDefinition> {
            let d = ArAgingDefinition { id: Uuid::new_v4(), organization_id: org_id, definition_code: code.into(), name: name.into(), description: desc.map(Into::into), aging_basis: basis.into(), num_buckets: nb, status: "active".into(), metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.defs.lock().unwrap().push(d.clone());
            Ok(d)
        }
        async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ArAgingDefinition>> { Ok(self.defs.lock().unwrap().iter().find(|d| d.organization_id == org_id && d.definition_code == code).cloned()) }
        async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<ArAgingDefinition>> { Ok(self.defs.lock().unwrap().iter().find(|d| d.id == id).cloned()) }
        async fn list_definitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ArAgingDefinition>> { Ok(self.defs.lock().unwrap().iter().filter(|d| d.organization_id == org_id && (status.is_none() || d.status == status.unwrap())).cloned().collect()) }
        async fn update_definition_status(&self, id: Uuid, status: &str) -> AtlasResult<ArAgingDefinition> {
            let mut ds = self.defs.lock().unwrap();
            let d = ds.iter_mut().find(|d| d.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            d.status = status.into();
            Ok(d.clone())
        }
        async fn delete_definition(&self, id: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn create_bucket(&self, org_id: Uuid, def_id: Uuid, bn: i32, name: &str, from: i32, to: Option<i32>, order: i32) -> AtlasResult<ArAgingBucket> {
            let b = ArAgingBucket { id: Uuid::new_v4(), organization_id: org_id, definition_id: def_id, bucket_number: bn, name: name.into(), from_days: from, to_days: to, display_order: order, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.buckets.lock().unwrap().push(b.clone());
            Ok(b)
        }
        async fn list_buckets(&self, def_id: Uuid) -> AtlasResult<Vec<ArAgingBucket>> { Ok(self.buckets.lock().unwrap().iter().filter(|b| b.definition_id == def_id).cloned().collect()) }
        async fn create_snapshot(&self, org_id: Uuid, def_id: Uuid, as_of: chrono::NaiveDate, cc: &str, cb: Option<Uuid>) -> AtlasResult<ArAgingSnapshot> {
            let s = ArAgingSnapshot { id: Uuid::new_v4(), organization_id: org_id, definition_id: def_id, snapshot_date: chrono::Utc::now().date_naive(), as_of_date: as_of, total_open_amount: "0".into(), total_overdue_amount: "0".into(), total_past_due_count: 0, currency_code: cc.into(), status: "completed".into(), metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.snapshots.lock().unwrap().push(s.clone());
            Ok(s)
        }
        async fn get_snapshot(&self, id: Uuid) -> AtlasResult<Option<ArAgingSnapshot>> { Ok(self.snapshots.lock().unwrap().iter().find(|s| s.id == id).cloned()) }
        async fn list_snapshots(&self, org_id: Uuid, def_id: Option<Uuid>) -> AtlasResult<Vec<ArAgingSnapshot>> { Ok(self.snapshots.lock().unwrap().iter().filter(|s| s.organization_id == org_id && (def_id.is_none() || s.definition_id == def_id.unwrap())).cloned().collect()) }
        async fn update_snapshot_totals(&self, _: Uuid, _: &str, _: &str, _: i32) -> AtlasResult<()> { Ok(()) }
        async fn create_snapshot_line(&self, org_id: Uuid, sid: Uuid, cid: Option<Uuid>, cn: &str, cname: Option<&str>, iid: Option<Uuid>, inum: &str, idate: chrono::NaiveDate, ddate: chrono::NaiveDate, oa: &str, open: &str, dpd: i32, bn: i32, bname: &str, cc: &str) -> AtlasResult<ArAgingSnapshotLine> {
            let l = ArAgingSnapshotLine { id: Uuid::new_v4(), organization_id: org_id, snapshot_id: sid, customer_id: cid, customer_number: cn.into(), customer_name: cname.map(Into::into), invoice_id: iid, invoice_number: inum.into(), invoice_date: idate, due_date: ddate, original_amount: oa.into(), open_amount: open.into(), days_past_due: dpd, bucket_number: bn, bucket_name: bname.into(), currency_code: cc.into(), metadata: serde_json::json!({}), created_at: chrono::Utc::now() };
            self.lines.lock().unwrap().push(l.clone());
            Ok(l)
        }
        async fn list_snapshot_lines(&self, sid: Uuid) -> AtlasResult<Vec<ArAgingSnapshotLine>> { Ok(self.lines.lock().unwrap().iter().filter(|l| l.snapshot_id == sid).cloned().collect()) }
        async fn get_aging_summary(&self, _: Uuid) -> AtlasResult<Vec<ArAgingSummary>> { Ok(vec![]) }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ArAgingDashboard> {
            let ds = self.defs.lock().unwrap();
            let ss = self.snapshots.lock().unwrap();
            Ok(ArAgingDashboard { total_definitions: ds.iter().filter(|d| d.organization_id == org_id).count() as i32, total_snapshots: ss.iter().filter(|s| s.organization_id == org_id).count() as i32, total_open_receivables: "0".into(), total_overdue: "0".into(), overdue_count: 0, avg_days_past_due: "0".into() })
        }
    }

    fn eng() -> ArAgingEngine { ArAgingEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_STATUSES.len(), 2);
        assert_eq!(VALID_AGING_BASES.len(), 3);
    }

    #[tokio::test]
    async fn test_create_definition_valid() {
        let d = eng().create_definition(Uuid::new_v4(), "STD-AGING", "Standard Aging", None, "due_date", 5, None).await.unwrap();
        assert_eq!(d.definition_code, "STD-AGING");
        assert_eq!(d.status, "active");
        assert_eq!(d.num_buckets, 5);
    }

    #[tokio::test]
    async fn test_create_definition_empty_code() {
        assert!(eng().create_definition(Uuid::new_v4(), "", "Name", None, "due_date", 5, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_empty_name() {
        assert!(eng().create_definition(Uuid::new_v4(), "CODE", "", None, "due_date", 5, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_invalid_basis() {
        assert!(eng().create_definition(Uuid::new_v4(), "CODE", "Name", None, "bad_basis", 5, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_zero_buckets() {
        assert!(eng().create_definition(Uuid::new_v4(), "CODE", "Name", None, "due_date", 0, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_too_many_buckets() {
        assert!(eng().create_definition(Uuid::new_v4(), "CODE", "Name", None, "due_date", 21, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_definition(org, "DUP", "T1", None, "due_date", 5, None).await;
        assert!(e.create_definition(org, "DUP", "T2", None, "due_date", 5, None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_definitions_invalid_status() {
        assert!(eng().list_definitions(Uuid::new_v4(), Some("bad")).await.is_err());
    }

    #[tokio::test]
    async fn test_create_bucket_valid() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let b = e.create_bucket(Uuid::new_v4(), d.id, 0, "Current", 0, Some(0), 0).await.unwrap();
        assert_eq!(b.name, "Current");
        assert_eq!(b.from_days, 0);
    }

    #[tokio::test]
    async fn test_create_bucket_empty_name() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        assert!(e.create_bucket(Uuid::new_v4(), d.id, 0, "", 0, Some(0), 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_bucket_negative_number() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        assert!(e.create_bucket(Uuid::new_v4(), d.id, -1, "Bucket", 0, Some(0), 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_bucket_negative_from_days() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        assert!(e.create_bucket(Uuid::new_v4(), d.id, 0, "Bucket", -1, Some(0), 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_bucket_to_before_from() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        assert!(e.create_bucket(Uuid::new_v4(), d.id, 0, "Bucket", 30, Some(10), 0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_bucket_open_ended() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let b = e.create_bucket(Uuid::new_v4(), d.id, 4, "91+", 91, None, 4).await.unwrap();
        assert_eq!(b.to_days, None);
    }

    #[tokio::test]
    async fn test_determine_bucket() {
        let e = eng();
        let buckets = vec![
            ArAgingBucket { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), definition_id: Uuid::new_v4(), bucket_number: 0, name: "Current".into(), from_days: 0, to_days: Some(0), display_order: 0, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() },
            ArAgingBucket { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), definition_id: Uuid::new_v4(), bucket_number: 1, name: "1-30".into(), from_days: 1, to_days: Some(30), display_order: 1, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() },
            ArAgingBucket { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), definition_id: Uuid::new_v4(), bucket_number: 2, name: "31-60".into(), from_days: 31, to_days: Some(60), display_order: 2, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() },
            ArAgingBucket { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), definition_id: Uuid::new_v4(), bucket_number: 3, name: "61-90".into(), from_days: 61, to_days: Some(90), display_order: 3, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() },
            ArAgingBucket { id: Uuid::new_v4(), organization_id: Uuid::new_v4(), definition_id: Uuid::new_v4(), bucket_number: 4, name: "91+".into(), from_days: 91, to_days: None, display_order: 4, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() },
        ];
        assert_eq!(e.determine_bucket(0, &buckets).unwrap().name, "Current");
        assert_eq!(e.determine_bucket(15, &buckets).unwrap().name, "1-30");
        assert_eq!(e.determine_bucket(45, &buckets).unwrap().name, "31-60");
        assert_eq!(e.determine_bucket(75, &buckets).unwrap().name, "61-90");
        assert_eq!(e.determine_bucket(120, &buckets).unwrap().name, "91+");
    }

    #[tokio::test]
    async fn test_create_snapshot_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let s = e.create_snapshot(org, d.id, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        assert_eq!(s.status, "completed");
    }

    #[tokio::test]
    async fn test_create_snapshot_bad_currency() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        assert!(e.create_snapshot(org, d.id, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "US", None).await.is_err());
    }

    #[tokio::test]
    async fn test_add_snapshot_line_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let s = e.create_snapshot(org, d.id, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let l = e.add_snapshot_line(org, s.id, None, "CUST-001", Some("Acme Corp"), None, "INV-100", chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(), "5000.00", "3000.00", 1, 1, "1-30", "USD").await.unwrap();
        assert_eq!(l.invoice_number, "INV-100");
        assert_eq!(l.open_amount, "3000.00");
    }

    #[tokio::test]
    async fn test_add_snapshot_line_empty_customer() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let s = e.create_snapshot(org, d.id, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        assert!(e.add_snapshot_line(org, s.id, None, "", None, None, "INV-100", chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(), "5000.00", "3000.00", 1, 1, "1-30", "USD").await.is_err());
    }

    #[tokio::test]
    async fn test_add_snapshot_line_negative_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let s = e.create_snapshot(org, d.id, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        assert!(e.add_snapshot_line(org, s.id, None, "CUST-1", None, None, "INV-100", chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(), "5000.00", "-100.00", 1, 1, "1-30", "USD").await.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create_definition(org, "STD", "Standard", None, "due_date", 5, None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_definitions, 1);
        assert_eq!(dash.total_snapshots, 0);
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        assert!(eng().delete_definition(Uuid::new_v4()).await.is_err());
    }
}
