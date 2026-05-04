//! Lockbox Processing Engine
//! Oracle Fusion: AR > Lockbox

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_BATCH_STATUSES: &[&str] = &["imported", "validated", "applied", "partial", "error", "completed"];
#[allow(dead_code)]
const VALID_RECEIPT_STATUSES: &[&str] = &["unapplied", "applied", "partial", "on_account", "error"];
#[allow(dead_code)]
const VALID_APPLICATION_STATUSES: &[&str] = &["applied", "unapplied", "reversed"];
#[allow(dead_code)]
const VALID_MATCH_TYPES: &[&str] = &["invoice_number", "customer", "auto", "manual"];
const VALID_FORMAT_TYPES: &[&str] = &["BAI2", "MT940", "OFX", "custom"];

pub struct LockboxEngine { repository: Arc<dyn LockboxRepository> }

impl LockboxEngine {
    pub fn new(r: Arc<dyn LockboxRepository>) -> Self { Self { repository: r } }

    // Batch operations
    pub async fn create_batch(&self, org_id: Uuid, batch_number: &str, lockbox_number: &str, bank_name: Option<&str>, deposit_date: chrono::NaiveDate, currency_code: &str, source_file_name: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<LockboxBatch> {
        if batch_number.is_empty() || lockbox_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Batch number and lockbox number are required".into()));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if self.repository.get_batch(org_id, batch_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Batch '{}' already exists", batch_number)));
        }
        info!("Creating lockbox batch {} for org {}", batch_number, org_id);
        self.repository.create_batch(org_id, batch_number, lockbox_number, bank_name, deposit_date, currency_code, source_file_name, created_by).await
    }

    pub async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<LockboxBatch>> { self.repository.get_batch_by_id(id).await }

    pub async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LockboxBatch>> {
        if let Some(s) = status { if !VALID_BATCH_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid batch status '{}'", s))); } }
        self.repository.list_batches(org_id, status).await
    }

    pub async fn validate_batch(&self, id: Uuid) -> AtlasResult<LockboxBatch> {
        let batch = self.repository.get_batch_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", id)))?;
        if batch.status != "imported" { return Err(AtlasError::WorkflowError(format!("Cannot validate batch in '{}' status", batch.status))); }
        let receipts = self.repository.list_receipts_by_batch(id).await?;
        let total: f64 = receipts.iter().map(|r| r.receipt_amount.parse::<f64>().unwrap_or(0.0)).sum();
        self.repository.update_batch_amounts(id, &total.to_string(), receipts.len() as i32, "0", &total.to_string(), "0").await?;
        self.repository.update_batch_status(id, "validated", None).await
    }

    pub async fn apply_batch(&self, id: Uuid) -> AtlasResult<LockboxBatch> {
        let batch = self.repository.get_batch_by_id(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", id)))?;
        if batch.status != "validated" { return Err(AtlasError::WorkflowError(format!("Cannot apply batch in '{}' status", batch.status))); }
        let receipts = self.repository.list_receipts_by_batch(id).await?;
        let mut applied: f64 = 0.0;
        let mut unapplied: f64 = 0.0;
        for receipt in &receipts {
            if receipt.status == "unapplied" {
                if let Some(_ref) = &receipt.remittance_reference {
                    applied += receipt.receipt_amount.parse::<f64>().unwrap_or(0.0);
                    self.repository.update_receipt_status(receipt.id, "applied", &receipt.receipt_amount, "0", "0", Some("auto"), None).await?;
                } else {
                    unapplied += receipt.receipt_amount.parse::<f64>().unwrap_or(0.0);
                }
            }
        }
        self.repository.update_batch_amounts(id, &batch.total_amount, receipts.len() as i32, &applied.to_string(), &unapplied.to_string(), "0").await?;
        let status = if unapplied > 0.0 { "partial" } else { "completed" };
        self.repository.update_batch_status(id, status, None).await
    }

    // Receipt operations
    pub async fn create_receipt(&self, org_id: Uuid, batch_id: Uuid, receipt_number: &str, customer_number: Option<&str>, customer_id: Option<Uuid>, receipt_date: chrono::NaiveDate, receipt_amount: &str, remittance_reference: Option<&str>) -> AtlasResult<LockboxReceipt> {
        let batch = self.repository.get_batch_by_id(batch_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;
        if batch.status != "imported" && batch.status != "validated" {
            return Err(AtlasError::WorkflowError(format!("Cannot add receipts to '{}' batch", batch.status)));
        }
        if receipt_number.is_empty() { return Err(AtlasError::ValidationFailed("Receipt number is required".into())); }
        let amt: f64 = receipt_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid receipt amount".into()))?;
        if amt <= 0.0 { return Err(AtlasError::ValidationFailed("Receipt amount must be positive".into())); }
        self.repository.create_receipt(org_id, batch_id, receipt_number, customer_number, customer_id, receipt_date, receipt_amount, remittance_reference).await
    }

    pub async fn list_receipts(&self, batch_id: Uuid) -> AtlasResult<Vec<LockboxReceipt>> {
        self.repository.list_receipts_by_batch(batch_id).await
    }

    pub async fn manual_apply_receipt(&self, receipt_id: Uuid, invoice_number: &str, applied_amount: &str, applied_by: Option<Uuid>) -> AtlasResult<LockboxApplication> {
        let receipt = self.repository.get_receipt(receipt_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", receipt_id)))?;
        if receipt.status != "unapplied" && receipt.status != "partial" {
            return Err(AtlasError::WorkflowError(format!("Cannot apply receipt in '{}' status", receipt.status)));
        }
        let amt: f64 = applied_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid applied amount".into()))?;
        if amt <= 0.0 { return Err(AtlasError::ValidationFailed("Applied amount must be positive".into())); }
        let receipt_amt: f64 = receipt.receipt_amount.parse().unwrap_or(0.0);
        let current_applied: f64 = receipt.applied_amount.parse().unwrap_or(0.0);
        if current_applied + amt > receipt_amt {
            return Err(AtlasError::ValidationFailed("Applied amount exceeds receipt amount".into()));
        }
        let app = self.repository.create_application(receipt.organization_id, receipt_id, None, Some(invoice_number), applied_amount, chrono::Utc::now().date_naive(), applied_by).await?;
        let new_applied = current_applied + amt;
        let new_unapplied = receipt_amt - new_applied;
        let new_status = if new_unapplied.abs() < 0.01 { "applied" } else { "partial" };
        self.repository.update_receipt_status(receipt_id, new_status, &new_applied.to_string(), &new_unapplied.to_string(), "0", Some("manual"), None).await?;
        Ok(app)
    }

    // Applications
    pub async fn list_applications(&self, receipt_id: Uuid) -> AtlasResult<Vec<LockboxApplication>> {
        self.repository.list_applications_by_receipt(receipt_id).await
    }

    pub async fn reverse_application(&self, app_id: Uuid) -> AtlasResult<LockboxApplication> {
        self.repository.reverse_application(app_id).await
    }

    // Transmission Formats
    pub async fn create_format(&self, org_id: Uuid, format_code: &str, name: &str, description: Option<&str>, format_type: &str, field_delimiter: Option<&str>, record_delimiter: Option<&str>, header_id: Option<&str>, detail_id: Option<&str>, trailer_id: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<LockboxTransmissionFormat> {
        if format_code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Format code and name are required".into()));
        }
        if !VALID_FORMAT_TYPES.contains(&format_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid format_type '{}'", format_type)));
        }
        self.repository.create_format(org_id, format_code, name, description, format_type, field_delimiter, record_delimiter, header_id, detail_id, trailer_id, created_by).await
    }

    pub async fn list_formats(&self, org_id: Uuid) -> AtlasResult<Vec<LockboxTransmissionFormat>> {
        self.repository.list_formats(org_id).await
    }

    pub async fn delete_format(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_format(id).await
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LockboxDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        batches: std::sync::Mutex<Vec<LockboxBatch>>,
        receipts: std::sync::Mutex<Vec<LockboxReceipt>>,
        applications: std::sync::Mutex<Vec<LockboxApplication>>,
    }
    impl MockRepo { fn new() -> Self { Self { batches: std::sync::Mutex::new(vec![]), receipts: std::sync::Mutex::new(vec![]), applications: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl LockboxRepository for MockRepo {
        async fn create_batch(&self, org_id: Uuid, bn: &str, ln: &str, bank: Option<&str>, dd: chrono::NaiveDate, cc: &str, sfn: Option<&str>, cb: Option<Uuid>) -> AtlasResult<LockboxBatch> {
            let b = LockboxBatch { id: Uuid::new_v4(), organization_id: org_id, batch_number: bn.into(), lockbox_number: ln.into(), bank_name: bank.map(Into::into), deposit_date: dd, status: "imported".into(), total_amount: "0".into(), total_receipts: 0, applied_amount: "0".into(), unapplied_amount: "0".into(), on_account_amount: "0".into(), currency_code: cc.into(), source_file_name: sfn.map(Into::into), error_message: None, metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.batches.lock().unwrap().push(b.clone());
            Ok(b)
        }
        async fn get_batch(&self, org_id: Uuid, bn: &str) -> AtlasResult<Option<LockboxBatch>> { Ok(self.batches.lock().unwrap().iter().find(|b| b.organization_id == org_id && b.batch_number == bn).cloned()) }
        async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<LockboxBatch>> { Ok(self.batches.lock().unwrap().iter().find(|b| b.id == id).cloned()) }
        async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LockboxBatch>> { Ok(self.batches.lock().unwrap().iter().filter(|b| b.organization_id == org_id && (status.is_none() || b.status == status.unwrap())).cloned().collect()) }
        async fn update_batch_status(&self, id: Uuid, status: &str, _: Option<&str>) -> AtlasResult<LockboxBatch> {
            let mut bs = self.batches.lock().unwrap();
            let b = bs.iter_mut().find(|b| b.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            b.status = status.into();
            Ok(b.clone())
        }
        async fn update_batch_amounts(&self, id: Uuid, total: &str, count: i32, applied: &str, unapplied: &str, on_account: &str) -> AtlasResult<()> {
            let mut bs = self.batches.lock().unwrap();
            if let Some(b) = bs.iter_mut().find(|b| b.id == id) { b.total_amount = total.into(); b.total_receipts = count; b.applied_amount = applied.into(); b.unapplied_amount = unapplied.into(); b.on_account_amount = on_account.into(); }
            Ok(())
        }
        async fn create_receipt(&self, org_id: Uuid, bid: Uuid, rn: &str, cn: Option<&str>, cid: Option<Uuid>, rd: chrono::NaiveDate, ra: &str, rr: Option<&str>) -> AtlasResult<LockboxReceipt> {
            let r = LockboxReceipt { id: Uuid::new_v4(), organization_id: org_id, batch_id: bid, receipt_number: rn.into(), customer_number: cn.map(Into::into), customer_id: cid, receipt_date: rd, receipt_amount: ra.into(), applied_amount: "0".into(), unapplied_amount: ra.into(), on_account_amount: "0".into(), status: "unapplied".into(), match_type: None, remittance_reference: rr.map(Into::into), error_message: None, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.receipts.lock().unwrap().push(r.clone());
            Ok(r)
        }
        async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<LockboxReceipt>> { Ok(self.receipts.lock().unwrap().iter().find(|r| r.id == id).cloned()) }
        async fn list_receipts_by_batch(&self, bid: Uuid) -> AtlasResult<Vec<LockboxReceipt>> { Ok(self.receipts.lock().unwrap().iter().filter(|r| r.batch_id == bid).cloned().collect()) }
        async fn update_receipt_status(&self, id: Uuid, status: &str, applied: &str, unapplied: &str, on_account: &str, mt: Option<&str>, _: Option<&str>) -> AtlasResult<LockboxReceipt> {
            let mut rs = self.receipts.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            r.status = status.into(); r.applied_amount = applied.into(); r.unapplied_amount = unapplied.into(); r.on_account_amount = on_account.into(); if let Some(m) = mt { r.match_type = Some(m.into()); }
            Ok(r.clone())
        }
        async fn create_application(&self, org_id: Uuid, rid: Uuid, iid: Option<Uuid>, inum: Option<&str>, amt: &str, ad: chrono::NaiveDate, ab: Option<Uuid>) -> AtlasResult<LockboxApplication> {
            let a = LockboxApplication { id: Uuid::new_v4(), organization_id: org_id, receipt_id: rid, invoice_id: iid, invoice_number: inum.map(Into::into), applied_amount: amt.into(), application_date: ad, status: "applied".into(), applied_by: ab, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.applications.lock().unwrap().push(a.clone());
            Ok(a)
        }
        async fn list_applications_by_receipt(&self, rid: Uuid) -> AtlasResult<Vec<LockboxApplication>> { Ok(self.applications.lock().unwrap().iter().filter(|a| a.receipt_id == rid).cloned().collect()) }
        async fn reverse_application(&self, id: Uuid) -> AtlasResult<LockboxApplication> { Err(AtlasError::EntityNotFound("Mock".into())) }
        async fn create_format(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<LockboxTransmissionFormat> { Err(AtlasError::DatabaseError("Not implemented".into())) }
        async fn list_formats(&self, _: Uuid) -> AtlasResult<Vec<LockboxTransmissionFormat>> { Ok(vec![]) }
        async fn delete_format(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<LockboxDashboard> { Ok(LockboxDashboard { total_batches: 0, pending_batches: 0, completed_batches: 0, total_receipts: 0, total_applied_amount: "0".into(), total_unapplied_amount: "0".into(), error_batches: 0 }) }
    }

    fn eng() -> LockboxEngine { LockboxEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_BATCH_STATUSES.len(), 6);
        assert_eq!(VALID_RECEIPT_STATUSES.len(), 5);
        assert_eq!(VALID_APPLICATION_STATUSES.len(), 3);
        assert_eq!(VALID_MATCH_TYPES.len(), 4);
        assert_eq!(VALID_FORMAT_TYPES.len(), 4);
    }

    #[tokio::test]
    async fn test_create_batch_valid() {
        let b = eng().create_batch(Uuid::new_v4(), "LB-001", "LB12345", Some("Chase"), chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", Some("lockbox.txt"), None).await.unwrap();
        assert_eq!(b.batch_number, "LB-001");
        assert_eq!(b.status, "imported");
    }

    #[tokio::test]
    async fn test_create_batch_empty_number() {
        assert!(eng().create_batch(Uuid::new_v4(), "", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_batch_empty_lockbox() {
        assert!(eng().create_batch(Uuid::new_v4(), "LB-1", "", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_batch_bad_currency() {
        assert!(eng().create_batch(Uuid::new_v4(), "LB-1", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "US", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_batch_duplicate() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_batch(org, "LB-DUP", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await;
        assert!(e.create_batch(org, "LB-DUP", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_batches_invalid_status() {
        assert!(eng().list_batches(Uuid::new_v4(), Some("bad")).await.is_err());
    }

    #[tokio::test]
    async fn test_validate_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-V", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        let r = e.create_receipt(org, b.id, "REC-1", Some("CUST-1"), None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "1000.00", Some("INV-001")).await.unwrap();
        let b = e.validate_batch(b.id).await.unwrap();
        assert_eq!(b.status, "validated");
    }

    #[tokio::test]
    async fn test_validate_batch_wrong_status() {
        let e = eng();
        let b = e.create_batch(Uuid::new_v4(), "LB-V2", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        let b = e.validate_batch(b.id).await.unwrap(); // imported -> validated
        assert!(e.validate_batch(b.id).await.is_err()); // can't re-validate
    }

    #[tokio::test]
    async fn test_apply_batch() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-A", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        let _ = e.create_receipt(org, b.id, "REC-1", Some("CUST-1"), None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "1000.00", Some("INV-001")).await.unwrap();
        let _ = e.validate_batch(b.id).await.unwrap();
        let b = e.apply_batch(b.id).await.unwrap();
        assert_eq!(b.status, "completed");
    }

    #[tokio::test]
    async fn test_apply_batch_unvalidated() {
        let e = eng();
        let b = e.create_batch(Uuid::new_v4(), "LB-A2", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        assert!(e.apply_batch(b.id).await.is_err());
    }

    #[tokio::test]
    async fn test_create_receipt_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-R", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        let r = e.create_receipt(org, b.id, "REC-1", Some("CUST-1"), None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "500.00", Some("INV-100")).await.unwrap();
        assert_eq!(r.receipt_number, "REC-1");
        assert_eq!(r.status, "unapplied");
    }

    #[tokio::test]
    async fn test_create_receipt_empty_number() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-R2", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        assert!(e.create_receipt(org, b.id, "", None, None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "500.00", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_receipt_zero_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-R3", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        assert!(e.create_receipt(org, b.id, "REC-1", None, None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "0", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_receipt_negative_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-R4", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        assert!(e.create_receipt(org, b.id, "REC-1", None, None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "-100.00", None).await.is_err());
    }

    #[tokio::test]
    async fn test_manual_apply_receipt() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-MA", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        let _ = e.create_receipt(org, b.id, "REC-1", Some("CUST-1"), None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "1000.00", None).await.unwrap();
        let r = e.manual_apply_receipt(e.list_receipts(b.id).await.unwrap()[0].id, "INV-100", "600.00", None).await.unwrap();
        assert_eq!(r.applied_amount, "600.00");
    }

    #[tokio::test]
    async fn test_manual_apply_receipt_exceeds_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_batch(org, "LB-MA2", "LB123", None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "USD", None, None).await.unwrap();
        let _ = e.create_receipt(org, b.id, "REC-1", Some("CUST-1"), None, chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), "500.00", None).await.unwrap();
        assert!(e.manual_apply_receipt(e.list_receipts(b.id).await.unwrap()[0].id, "INV-100", "600.00", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_format_invalid_type() {
        assert!(eng().create_format(Uuid::new_v4(), "F1", "Format", None, "CSV", None, None, None, None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_format_empty_code() {
        assert!(eng().create_format(Uuid::new_v4(), "", "Format", None, "BAI2", None, None, None, None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let dash = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(dash.total_batches, 0);
    }
}
