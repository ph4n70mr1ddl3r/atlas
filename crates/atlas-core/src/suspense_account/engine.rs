//! Suspense Account Processing Engine
//! Oracle Fusion: General Ledger > Suspense Accounts

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_ENTRY_TYPES: &[&str] = &["auto", "manual"];
const VALID_ENTRY_STATUSES: &[&str] = &["open", "cleared", "written_off", "reversed"];
const VALID_BATCH_STATUSES: &[&str] = &["draft", "submitted", "approved", "posted", "reversed"];
#[allow(dead_code)]
const VALID_DEFINITION_STATUSES: &[&str] = &["active", "inactive"];

pub struct SuspenseAccountEngine { repository: Arc<dyn SuspenseAccountRepository> }

impl SuspenseAccountEngine {
    pub fn new(r: Arc<dyn SuspenseAccountRepository>) -> Self { Self { repository: r } }

    // ── Definition CRUD ──────────────────────────────────────

    pub async fn create_definition(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        balancing_segment: &str, suspense_account: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<SuspenseAccountDefinition> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if balancing_segment.is_empty() {
            return Err(AtlasError::ValidationFailed("Balancing segment is required".into()));
        }
        if suspense_account.is_empty() {
            return Err(AtlasError::ValidationFailed("Suspense account is required".into()));
        }
        if self.repository.get_definition_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Suspense definition '{}' already exists", code)));
        }
        info!("Creating suspense account definition {} for org {}", code, org_id);
        self.repository.create_definition(org_id, code, name, description, balancing_segment, suspense_account, created_by).await
    }

    pub async fn get_definition(&self, id: Uuid) -> AtlasResult<Option<SuspenseAccountDefinition>> {
        self.repository.get_definition(id).await
    }

    pub async fn list_definitions(&self, org_id: Uuid) -> AtlasResult<Vec<SuspenseAccountDefinition>> {
        self.repository.list_definitions(org_id).await
    }

    pub async fn activate_definition(&self, id: Uuid) -> AtlasResult<SuspenseAccountDefinition> {
        let def = self.repository.get_definition(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", id)))?;
        if def.enabled && def.status == "active" {
            return Err(AtlasError::WorkflowError("Definition is already active".into()));
        }
        self.repository.update_definition_status(id, true, "active").await
    }

    pub async fn deactivate_definition(&self, id: Uuid) -> AtlasResult<SuspenseAccountDefinition> {
        let def = self.repository.get_definition(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", id)))?;
        if !def.enabled || def.status == "inactive" {
            return Err(AtlasError::WorkflowError("Definition is already inactive".into()));
        }
        // Check for open entries
        let open_entries = self.repository.list_entries_by_definition(id, Some("open")).await?;
        if !open_entries.is_empty() {
            return Err(AtlasError::WorkflowError(format!("Cannot deactivate: {} open entries exist", open_entries.len())));
        }
        self.repository.update_definition_status(id, false, "inactive").await
    }

    pub async fn delete_definition(&self, id: Uuid) -> AtlasResult<()> {
        let def = self.repository.get_definition(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", id)))?;
        if def.status == "active" {
            return Err(AtlasError::WorkflowError("Cannot delete an active definition. Deactivate first.".into()));
        }
        self.repository.delete_definition(id).await
    }

    // ── Suspense Entry Operations ────────────────────────────

    pub async fn create_entry(
        &self, org_id: Uuid, definition_id: Uuid, journal_entry_id: Option<Uuid>,
        journal_batch_id: Option<Uuid>, balancing_segment_value: &str,
        suspense_amount: &str, original_amount: Option<&str>,
        entry_type: &str, entry_date: chrono::NaiveDate, currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SuspenseEntry> {
        let def = self.repository.get_definition(definition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Definition {} not found", definition_id)))?;
        if !def.enabled || def.status != "active" {
            return Err(AtlasError::WorkflowError("Definition is not active".into()));
        }
        if !VALID_ENTRY_TYPES.contains(&entry_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid entry_type '{}'", entry_type)));
        }
        if balancing_segment_value.is_empty() {
            return Err(AtlasError::ValidationFailed("Balancing segment value is required".into()));
        }
        let amt: f64 = suspense_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid suspense amount".into()))?;
        if amt == 0.0 {
            return Err(AtlasError::ValidationFailed("Suspense amount cannot be zero".into()));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        info!("Creating suspense entry for definition {} amount {}", def.code, suspense_amount);
        self.repository.create_entry(
            org_id, definition_id, journal_entry_id, journal_batch_id,
            balancing_segment_value, &def.suspense_account, suspense_amount,
            original_amount, entry_type, entry_date, currency_code, created_by,
        ).await
    }

    pub async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<SuspenseEntry>> {
        self.repository.get_entry(id).await
    }

    pub async fn list_entries(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SuspenseEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid entry status '{}'", s)));
            }
        }
        self.repository.list_entries(org_id, status).await
    }

    pub async fn list_entries_by_definition(&self, definition_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SuspenseEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid entry status '{}'", s)));
            }
        }
        self.repository.list_entries_by_definition(definition_id, status).await
    }

    pub async fn reverse_entry(&self, id: Uuid, resolution_notes: Option<&str>) -> AtlasResult<SuspenseEntry> {
        let entry = self.repository.get_entry(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", id)))?;
        if entry.status != "open" {
            return Err(AtlasError::WorkflowError(format!("Cannot reverse entry in '{}' status", entry.status)));
        }
        self.repository.update_entry_status(id, "reversed", None, Some(chrono::Utc::now().date_naive()), resolution_notes).await
    }

    pub async fn write_off_entry(&self, id: Uuid, resolution_notes: Option<&str>) -> AtlasResult<SuspenseEntry> {
        let entry = self.repository.get_entry(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", id)))?;
        if entry.status != "open" {
            return Err(AtlasError::WorkflowError(format!("Cannot write off entry in '{}' status", entry.status)));
        }
        if resolution_notes.is_none() || resolution_notes.unwrap().is_empty() {
            return Err(AtlasError::ValidationFailed("Resolution notes are required for write-off".into()));
        }
        self.repository.update_entry_status(id, "written_off", None, Some(chrono::Utc::now().date_naive()), resolution_notes).await
    }

    // ── Clearing Operations ──────────────────────────────────

    pub async fn create_clearing_batch(
        &self, org_id: Uuid, batch_number: &str, description: Option<&str>,
        clearing_date: chrono::NaiveDate, created_by: Option<Uuid>,
    ) -> AtlasResult<SuspenseClearingBatch> {
        if batch_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Batch number is required".into()));
        }
        if self.repository.get_clearing_batch_by_number(org_id, batch_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Clearing batch '{}' already exists", batch_number)));
        }
        info!("Creating suspense clearing batch {} for org {}", batch_number, org_id);
        self.repository.create_clearing_batch(org_id, batch_number, description, clearing_date, created_by).await
    }

    pub async fn get_clearing_batch(&self, id: Uuid) -> AtlasResult<Option<SuspenseClearingBatch>> {
        self.repository.get_clearing_batch(id).await
    }

    pub async fn list_clearing_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SuspenseClearingBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid batch status '{}'", s)));
            }
        }
        self.repository.list_clearing_batches(org_id, status).await
    }

    pub async fn add_clearing_line(
        &self, org_id: Uuid, batch_id: Uuid, entry_id: Uuid,
        clearing_account: &str, cleared_amount: &str, resolution_notes: Option<&str>,
    ) -> AtlasResult<SuspenseClearingLine> {
        let batch = self.repository.get_clearing_batch(batch_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;
        if batch.status != "draft" && batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!("Cannot add lines to '{}' batch", batch.status)));
        }
        let entry = self.repository.get_entry(entry_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", entry_id)))?;
        if entry.status != "open" {
            return Err(AtlasError::WorkflowError(format!("Entry {} is not open", entry_id)));
        }
        if clearing_account.is_empty() {
            return Err(AtlasError::ValidationFailed("Clearing account is required".into()));
        }
        let amt: f64 = cleared_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid cleared amount".into()))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Cleared amount must be positive".into()));
        }
        let entry_amt: f64 = entry.suspense_amount.parse().unwrap_or(0.0);
        if amt > entry_amt.abs() {
            return Err(AtlasError::ValidationFailed("Cleared amount exceeds suspense amount".into()));
        }
        self.repository.create_clearing_line(org_id, batch_id, entry_id, clearing_account, cleared_amount, resolution_notes).await
    }

    pub async fn list_clearing_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<SuspenseClearingLine>> {
        self.repository.list_clearing_lines(batch_id).await
    }

    pub async fn submit_clearing_batch(&self, id: Uuid) -> AtlasResult<SuspenseClearingBatch> {
        let batch = self.repository.get_clearing_batch(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", id)))?;
        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(format!("Cannot submit batch in '{}' status", batch.status)));
        }
        let lines = self.repository.list_clearing_lines(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed("Cannot submit an empty clearing batch".into()));
        }
        let total: f64 = lines.iter().map(|l| l.cleared_amount.parse::<f64>().unwrap_or(0.0)).sum();
        self.repository.update_clearing_batch(id, "submitted", lines.len() as i32, &total.to_string()).await
    }

    pub async fn approve_clearing_batch(&self, id: Uuid) -> AtlasResult<SuspenseClearingBatch> {
        let batch = self.repository.get_clearing_batch(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", id)))?;
        if batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!("Cannot approve batch in '{}' status", batch.status)));
        }
        // Mark all entries as cleared
        let lines = self.repository.list_clearing_lines(id).await?;
        let clearing_date = batch.clearing_date;
        for line in &lines {
            let _ = self.repository.update_entry_status(
                line.suspense_entry_id, "cleared", None,
                Some(clearing_date), line.resolution_notes.as_deref(),
            ).await;
        }
        self.repository.update_clearing_batch(id, "approved", batch.total_entries, &batch.total_cleared_amount).await
    }

    pub async fn post_clearing_batch(&self, id: Uuid) -> AtlasResult<SuspenseClearingBatch> {
        let batch = self.repository.get_clearing_batch(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", id)))?;
        if batch.status != "approved" {
            return Err(AtlasError::WorkflowError(format!("Cannot post batch in '{}' status", batch.status)));
        }
        self.repository.update_clearing_batch(id, "posted", batch.total_entries, &batch.total_cleared_amount).await
    }

    // ── Aging & Dashboard ────────────────────────────────────

    pub async fn create_aging_snapshot(&self, org_id: Uuid, snapshot_date: chrono::NaiveDate) -> AtlasResult<SuspenseAgingSnapshot> {
        info!("Creating suspense aging snapshot for org {} on {}", org_id, snapshot_date);
        self.repository.create_aging_snapshot(org_id, snapshot_date).await
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SuspenseDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        definitions: std::sync::Mutex<Vec<SuspenseAccountDefinition>>,
        entries: std::sync::Mutex<Vec<SuspenseEntry>>,
        clearing_batches: std::sync::Mutex<Vec<SuspenseClearingBatch>>,
        clearing_lines: std::sync::Mutex<Vec<SuspenseClearingLine>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                definitions: std::sync::Mutex::new(vec![]),
                entries: std::sync::Mutex::new(vec![]),
                clearing_batches: std::sync::Mutex::new(vec![]),
                clearing_lines: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl SuspenseAccountRepository for MockRepo {
        async fn create_definition(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, seg: &str, acct: &str, cb: Option<Uuid>) -> AtlasResult<SuspenseAccountDefinition> {
            let d = SuspenseAccountDefinition {
                id: Uuid::new_v4(), organization_id: org_id, code: code.into(), name: name.into(),
                description: desc.map(Into::into), balancing_segment: seg.into(), suspense_account: acct.into(),
                enabled: true, status: "active".into(), metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.definitions.lock().unwrap().push(d.clone());
            Ok(d)
        }
        async fn get_definition(&self, id: Uuid) -> AtlasResult<Option<SuspenseAccountDefinition>> {
            Ok(self.definitions.lock().unwrap().iter().find(|d| d.id == id).cloned())
        }
        async fn get_definition_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SuspenseAccountDefinition>> {
            Ok(self.definitions.lock().unwrap().iter().find(|d| d.organization_id == org_id && d.code == code).cloned())
        }
        async fn list_definitions(&self, org_id: Uuid) -> AtlasResult<Vec<SuspenseAccountDefinition>> {
            Ok(self.definitions.lock().unwrap().iter().filter(|d| d.organization_id == org_id).cloned().collect())
        }
        async fn update_definition_status(&self, id: Uuid, enabled: bool, status: &str) -> AtlasResult<SuspenseAccountDefinition> {
            let mut ds = self.definitions.lock().unwrap();
            let d = ds.iter_mut().find(|d| d.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            d.enabled = enabled; d.status = status.into();
            Ok(d.clone())
        }
        async fn delete_definition(&self, id: Uuid) -> AtlasResult<()> {
            let mut ds = self.definitions.lock().unwrap();
            ds.retain(|d| d.id != id);
            Ok(())
        }
        async fn create_entry(&self, org_id: Uuid, def_id: Uuid, je_id: Option<Uuid>, jb_id: Option<Uuid>, bsv: &str, sa: &str, amt: &str, oa: Option<&str>, et: &str, ed: chrono::NaiveDate, cc: &str, cb: Option<Uuid>) -> AtlasResult<SuspenseEntry> {
            let e = SuspenseEntry {
                id: Uuid::new_v4(), organization_id: org_id, suspense_definition_id: def_id,
                journal_entry_id: je_id, journal_batch_id: jb_id, balancing_segment_value: bsv.into(),
                suspense_account: sa.into(), suspense_amount: amt.into(), original_amount: oa.map(Into::into),
                entry_type: et.into(), entry_date: ed, currency_code: cc.into(), status: "open".into(),
                cleared_by_journal_id: None, clearing_date: None, resolution_notes: None,
                metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.entries.lock().unwrap().push(e.clone());
            Ok(e)
        }
        async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<SuspenseEntry>> {
            Ok(self.entries.lock().unwrap().iter().find(|e| e.id == id).cloned())
        }
        async fn list_entries(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SuspenseEntry>> {
            Ok(self.entries.lock().unwrap().iter().filter(|e| e.organization_id == org_id && (status.is_none() || e.status == status.unwrap())).cloned().collect())
        }
        async fn list_entries_by_definition(&self, def_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SuspenseEntry>> {
            Ok(self.entries.lock().unwrap().iter().filter(|e| e.suspense_definition_id == def_id && (status.is_none() || e.status == status.unwrap())).cloned().collect())
        }
        async fn update_entry_status(&self, id: Uuid, status: &str, cjid: Option<Uuid>, cd: Option<chrono::NaiveDate>, rn: Option<&str>) -> AtlasResult<SuspenseEntry> {
            let mut es = self.entries.lock().unwrap();
            let e = es.iter_mut().find(|e| e.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            e.status = status.into(); e.cleared_by_journal_id = cjid; e.clearing_date = cd; e.resolution_notes = rn.map(Into::into);
            Ok(e.clone())
        }
        async fn create_clearing_batch(&self, org_id: Uuid, bn: &str, desc: Option<&str>, cd: chrono::NaiveDate, cb: Option<Uuid>) -> AtlasResult<SuspenseClearingBatch> {
            let b = SuspenseClearingBatch {
                id: Uuid::new_v4(), organization_id: org_id, batch_number: bn.into(),
                description: desc.map(Into::into), clearing_date: cd, status: "draft".into(),
                total_entries: 0, total_cleared_amount: "0".into(), metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.clearing_batches.lock().unwrap().push(b.clone());
            Ok(b)
        }
        async fn get_clearing_batch(&self, id: Uuid) -> AtlasResult<Option<SuspenseClearingBatch>> {
            Ok(self.clearing_batches.lock().unwrap().iter().find(|b| b.id == id).cloned())
        }
        async fn get_clearing_batch_by_number(&self, org_id: Uuid, bn: &str) -> AtlasResult<Option<SuspenseClearingBatch>> {
            Ok(self.clearing_batches.lock().unwrap().iter().find(|b| b.organization_id == org_id && b.batch_number == bn).cloned())
        }
        async fn list_clearing_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SuspenseClearingBatch>> {
            Ok(self.clearing_batches.lock().unwrap().iter().filter(|b| b.organization_id == org_id && (status.is_none() || b.status == status.unwrap())).cloned().collect())
        }
        async fn update_clearing_batch(&self, id: Uuid, status: &str, te: i32, tca: &str) -> AtlasResult<SuspenseClearingBatch> {
            let mut bs = self.clearing_batches.lock().unwrap();
            let b = bs.iter_mut().find(|b| b.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            b.status = status.into(); b.total_entries = te; b.total_cleared_amount = tca.into();
            Ok(b.clone())
        }
        async fn create_clearing_line(&self, org_id: Uuid, batch_id: Uuid, entry_id: Uuid, ca: &str, amt: &str, rn: Option<&str>) -> AtlasResult<SuspenseClearingLine> {
            let l = SuspenseClearingLine {
                id: Uuid::new_v4(), organization_id: org_id, clearing_batch_id: batch_id,
                suspense_entry_id: entry_id, clearing_account: ca.into(), cleared_amount: amt.into(),
                status: "pending".into(), resolution_notes: rn.map(Into::into), metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.clearing_lines.lock().unwrap().push(l.clone());
            Ok(l)
        }
        async fn list_clearing_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<SuspenseClearingLine>> {
            Ok(self.clearing_lines.lock().unwrap().iter().filter(|l| l.clearing_batch_id == batch_id).cloned().collect())
        }
        async fn create_aging_snapshot(&self, org_id: Uuid, sd: chrono::NaiveDate) -> AtlasResult<SuspenseAgingSnapshot> {
            Ok(SuspenseAgingSnapshot {
                id: Uuid::new_v4(), organization_id: org_id, snapshot_date: sd,
                total_open_entries: 0, total_open_amount: "0".into(),
                aging_0_30: "0".into(), aging_31_60: "0".into(), aging_61_90: "0".into(),
                aging_91_180: "0".into(), aging_over_180: "0".into(),
                metadata: serde_json::json!({}), created_at: chrono::Utc::now(),
            })
        }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<SuspenseDashboard> {
            Ok(SuspenseDashboard {
                total_definitions: 0, active_definitions: 0,
                total_open_entries: 0, total_open_amount: "0".into(),
                total_cleared_entries: 0, total_cleared_amount: "0".into(),
                oldest_open_entry_days: 0,
            })
        }
    }

    fn eng() -> SuspenseAccountEngine { SuspenseAccountEngine::new(Arc::new(MockRepo::new())) }

    // ── Definition Tests ─────────────────────────────────────

    #[tokio::test]
    async fn test_create_definition_valid() {
        let d = eng().create_definition(
            Uuid::new_v4(), "SUSP-01", "Primary Suspense", Some("Main suspense account"),
            "company", "9999-000-0001", None,
        ).await.unwrap();
        assert_eq!(d.code, "SUSP-01");
        assert_eq!(d.status, "active");
        assert!(d.enabled);
    }

    #[tokio::test]
    async fn test_create_definition_empty_code() {
        let r = eng().create_definition(Uuid::new_v4(), "", "Name", None, "seg", "acct", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_empty_name() {
        let r = eng().create_definition(Uuid::new_v4(), "CODE", "", None, "seg", "acct", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_empty_segment() {
        let r = eng().create_definition(Uuid::new_v4(), "CODE", "Name", None, "", "acct", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_empty_account() {
        let r = eng().create_definition(Uuid::new_v4(), "CODE", "Name", None, "seg", "", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_definition_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_definition(org, "DUP", "Name", None, "seg", "acct", None).await;
        let r = e.create_definition(org, "DUP", "Name", None, "seg", "acct", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_definition() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "DEACT", "Name", None, "seg", "acct", None).await.unwrap();
        let d = e.deactivate_definition(d.id).await.unwrap();
        assert!(!d.enabled);
        assert_eq!(d.status, "inactive");
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "DEACT2", "Name", None, "seg", "acct", None).await.unwrap();
        let _ = e.deactivate_definition(d.id).await.unwrap();
        assert!(e.deactivate_definition(d.id).await.is_err());
    }

    #[tokio::test]
    async fn test_activate_definition() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "ACT", "Name", None, "seg", "acct", None).await.unwrap();
        let _ = e.deactivate_definition(d.id).await.unwrap();
        let d = e.activate_definition(d.id).await.unwrap();
        assert!(d.enabled);
        assert_eq!(d.status, "active");
    }

    #[tokio::test]
    async fn test_activate_already_active() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "ACT2", "Name", None, "seg", "acct", None).await.unwrap();
        assert!(e.activate_definition(d.id).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_active_definition() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "DEL1", "Name", None, "seg", "acct", None).await.unwrap();
        assert!(e.delete_definition(d.id).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_inactive_definition() {
        let e = eng();
        let d = e.create_definition(Uuid::new_v4(), "DEL2", "Name", None, "seg", "acct", None).await.unwrap();
        let _ = e.deactivate_definition(d.id).await.unwrap();
        assert!(e.delete_definition(d.id).await.is_ok());
    }

    #[tokio::test]
    async fn test_list_definitions() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_definition(org, "L1", "Name 1", None, "seg", "acct", None).await;
        let _ = e.create_definition(org, "L2", "Name 2", None, "seg", "acct", None).await;
        let list = e.list_definitions(org).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    // ── Entry Tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_create_entry_auto() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(
            org, d.id, None, None, "US01", "1500.00", Some("10000.00"),
            "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None,
        ).await.unwrap();
        assert_eq!(entry.status, "open");
        assert_eq!(entry.entry_type, "auto");
        assert_eq!(entry.suspense_amount, "1500.00");
    }

    #[tokio::test]
    async fn test_create_entry_manual() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E2", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(
            org, d.id, None, None, "US01", "500.00", None,
            "manual", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None,
        ).await.unwrap();
        assert_eq!(entry.entry_type, "manual");
    }

    #[tokio::test]
    async fn test_create_entry_invalid_type() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E3", "Name", None, "company", "9999-000", None).await.unwrap();
        let r = e.create_entry(org, d.id, None, None, "US01", "100", None, "invalid", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_zero_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E4", "Name", None, "company", "9999-000", None).await.unwrap();
        let r = e.create_entry(org, d.id, None, None, "US01", "0", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_empty_segment() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E5", "Name", None, "company", "9999-000", None).await.unwrap();
        let r = e.create_entry(org, d.id, None, None, "", "100", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_bad_currency() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E6", "Name", None, "company", "9999-000", None).await.unwrap();
        let r = e.create_entry(org, d.id, None, None, "US01", "100", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "US", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_inactive_definition() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "E7", "Name", None, "company", "9999-000", None).await.unwrap();
        let _ = e.deactivate_definition(d.id).await.unwrap();
        let r = e.create_entry(org, d.id, None, None, "US01", "100", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_reverse_entry() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "R1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, d.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let entry = e.reverse_entry(entry.id, Some("Reversed in error")).await.unwrap();
        assert_eq!(entry.status, "reversed");
    }

    #[tokio::test]
    async fn test_reverse_non_open_entry() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "R2", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, d.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let _ = e.reverse_entry(entry.id, Some("Reversed")).await.unwrap();
        assert!(e.reverse_entry(entry.id, Some("Again")).await.is_err());
    }

    #[tokio::test]
    async fn test_write_off_entry() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "W1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, d.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let entry = e.write_off_entry(entry.id, Some("Immaterial amount")).await.unwrap();
        assert_eq!(entry.status, "written_off");
    }

    #[tokio::test]
    async fn test_write_off_without_notes() {
        let e = eng();
        let org = Uuid::new_v4();
        let d = e.create_definition(org, "W2", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, d.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        assert!(e.write_off_entry(entry.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_entries_filter_status() {
        let e = eng();
        let org = Uuid::new_v4();
        let list = e.list_entries(org, Some("open")).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_list_entries_invalid_status() {
        assert!(eng().list_entries(Uuid::new_v4(), Some("bad")).await.is_err());
    }

    // ── Clearing Batch Tests ─────────────────────────────────

    #[tokio::test]
    async fn test_create_clearing_batch() {
        let b = eng().create_clearing_batch(
            Uuid::new_v4(), "CLB-001", Some("Monthly clearing"),
            chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None,
        ).await.unwrap();
        assert_eq!(b.batch_number, "CLB-001");
        assert_eq!(b.status, "draft");
    }

    #[tokio::test]
    async fn test_create_clearing_batch_empty_number() {
        let r = eng().create_clearing_batch(
            Uuid::new_v4(), "", None,
            chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_clearing_batch_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_clearing_batch(org, "DUP", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None).await;
        assert!(e.create_clearing_batch(org, "DUP", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_submit_clearing_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let def = e.create_definition(org, "S1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, def.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let batch = e.create_clearing_batch(org, "CLB-S", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(), None).await.unwrap();
        let _ = e.add_clearing_line(org, batch.id, entry.id, "4100-000", "500", Some("Cleared")).await.unwrap();
        let batch = e.submit_clearing_batch(batch.id).await.unwrap();
        assert_eq!(batch.status, "submitted");
        assert_eq!(batch.total_entries, 1);
    }

    #[tokio::test]
    async fn test_submit_empty_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let batch = e.create_clearing_batch(org, "CLB-E", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None).await.unwrap();
        assert!(e.submit_clearing_batch(batch.id).await.is_err());
    }

    #[tokio::test]
    async fn test_approve_clearing_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let def = e.create_definition(org, "A1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, def.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let batch = e.create_clearing_batch(org, "CLB-A", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(), None).await.unwrap();
        let _ = e.add_clearing_line(org, batch.id, entry.id, "4100-000", "500", None).await.unwrap();
        let _ = e.submit_clearing_batch(batch.id).await.unwrap();
        let batch = e.approve_clearing_batch(batch.id).await.unwrap();
        assert_eq!(batch.status, "approved");
        // Verify entry was cleared
        let entry = e.get_entry(entry.id).await.unwrap().unwrap();
        assert_eq!(entry.status, "cleared");
    }

    #[tokio::test]
    async fn test_approve_unsubmitted_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let batch = e.create_clearing_batch(org, "CLB-U", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None).await.unwrap();
        assert!(e.approve_clearing_batch(batch.id).await.is_err());
    }

    #[tokio::test]
    async fn test_post_clearing_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let def = e.create_definition(org, "P1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, def.id, None, None, "US01", "500", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let batch = e.create_clearing_batch(org, "CLB-P", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(), None).await.unwrap();
        let _ = e.add_clearing_line(org, batch.id, entry.id, "4100-000", "500", None).await.unwrap();
        let _ = e.submit_clearing_batch(batch.id).await.unwrap();
        let _ = e.approve_clearing_batch(batch.id).await.unwrap();
        let batch = e.post_clearing_batch(batch.id).await.unwrap();
        assert_eq!(batch.status, "posted");
    }

    #[tokio::test]
    async fn test_post_unapproved_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let batch = e.create_clearing_batch(org, "CLB-PU", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), None).await.unwrap();
        assert!(e.post_clearing_batch(batch.id).await.is_err());
    }

    #[tokio::test]
    async fn test_add_clearing_line_exceeds_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let def = e.create_definition(org, "X1", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, def.id, None, None, "US01", "200", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let batch = e.create_clearing_batch(org, "CLB-X", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(), None).await.unwrap();
        let r = e.add_clearing_line(org, batch.id, entry.id, "4100-000", "300", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_add_clearing_line_zero_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let def = e.create_definition(org, "X2", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, def.id, None, None, "US01", "200", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let batch = e.create_clearing_batch(org, "CLB-X2", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(), None).await.unwrap();
        let r = e.add_clearing_line(org, batch.id, entry.id, "4100-000", "0", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_add_clearing_line_to_posted_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let def = e.create_definition(org, "X3", "Name", None, "company", "9999-000", None).await.unwrap();
        let entry = e.create_entry(org, def.id, None, None, "US01", "200", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None).await.unwrap();
        let batch = e.create_clearing_batch(org, "CLB-X3", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 3).unwrap(), None).await.unwrap();
        let _ = e.add_clearing_line(org, batch.id, entry.id, "4100-000", "200", None).await.unwrap();
        let _ = e.submit_clearing_batch(batch.id).await.unwrap();
        let _ = e.approve_clearing_batch(batch.id).await.unwrap();
        let _ = e.post_clearing_batch(batch.id).await.unwrap();
        // Try to add a line to a posted batch
        let entry2 = e.create_entry(org, def.id, None, None, "US02", "100", None, "auto", chrono::NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(), "USD", None).await.unwrap();
        assert!(e.add_clearing_line(org, batch.id, entry2.id, "4100-000", "100", None).await.is_err());
    }

    // ── Dashboard / Aging Tests ──────────────────────────────

    #[tokio::test]
    async fn test_get_dashboard() {
        let dash = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(dash.total_definitions, 0);
        assert_eq!(dash.total_open_entries, 0);
    }

    #[tokio::test]
    async fn test_create_aging_snapshot() {
        let snap = eng().create_aging_snapshot(Uuid::new_v4(), chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()).await.unwrap();
        assert_eq!(snap.total_open_entries, 0);
    }

    #[tokio::test]
    async fn test_list_clearing_batches_invalid_status() {
        assert!(eng().list_clearing_batches(Uuid::new_v4(), Some("bad")).await.is_err());
    }
}
