//! Payment Format Module
//!
//! Oracle Fusion Cloud ERP-inspired Payment Format Management.
//! Configures payment file formats for various payment methods
//! (checks, electronic funds transfers, wire transfers, etc.)
//!
//! Oracle Fusion equivalent: Financials > Payables > Payment Formats

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFormat {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub format_type: String,
    pub payment_method: String,
    pub file_template: Option<String>,
    pub requires_bank_details: bool,
    pub supports_remittance: bool,
    pub supports_void: bool,
    pub max_payments_per_file: Option<i32>,
    pub currency_code: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFormatDashboard {
    pub organization_id: Uuid,
    pub total_formats: i32,
    pub active_formats: i32,
    pub by_format_type: serde_json::Value,
    pub by_payment_method: serde_json::Value,
}

// ============================================================================
// Constants
// ============================================================================

const VALID_FORMAT_TYPES: &[&str] = &[
    "file", "printed_check", "edi", "xml", "json", "swift", "bacl2", "pain001",
];

const VALID_PAYMENT_METHODS: &[&str] = &[
    "check", "eft", "wire", "ach", "sepa", "bacs", "swift_transfer", "clearing",
];

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait PaymentFormatRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        format_type: &str, payment_method: &str, file_template: Option<&str>,
        requires_bank_details: bool, supports_remittance: bool, supports_void: bool,
        max_payments_per_file: Option<i32>, currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentFormat>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<PaymentFormat>>;
    async fn get_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentFormat>>;
    async fn list(&self, org_id: Uuid, format_type: Option<&str>, payment_method: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<PaymentFormat>>;
    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>, file_template: Option<&str>, max_payments: Option<i32>) -> AtlasResult<PaymentFormat>;
    async fn deactivate(&self, id: Uuid) -> AtlasResult<PaymentFormat>;
    async fn activate(&self, id: Uuid) -> AtlasResult<PaymentFormat>;
    async fn delete(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentFormatDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresPaymentFormatRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresPaymentFormatRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl PaymentFormatRepository for PostgresPaymentFormatRepository {
    async fn create(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: bool, _: bool, _: bool, _: Option<i32>, _: &str, _: Option<Uuid>) -> AtlasResult<PaymentFormat> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<PaymentFormat>> { Ok(None) }
    async fn get_by_code(&self, _: Uuid, _: &str) -> AtlasResult<Option<PaymentFormat>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<&str>, _: Option<&str>, _: Option<bool>) -> AtlasResult<Vec<PaymentFormat>> { Ok(vec![]) }
    async fn update(&self, _: Uuid, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<i32>) -> AtlasResult<PaymentFormat> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn deactivate(&self, _: Uuid) -> AtlasResult<PaymentFormat> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn activate(&self, _: Uuid) -> AtlasResult<PaymentFormat> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn delete(&self, _: Uuid, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentFormatDashboard> {
        Ok(PaymentFormatDashboard {
            organization_id: org_id, total_formats: 0, active_formats: 0,
            by_format_type: serde_json::json!([]), by_payment_method: serde_json::json!([]),
        })
    }
}

// ============================================================================
// Engine
// ============================================================================

use std::sync::Arc;
use tracing::info;

pub struct PaymentFormatEngine {
    repository: Arc<dyn PaymentFormatRepository>,
}

impl PaymentFormatEngine {
    pub fn new(repository: Arc<dyn PaymentFormatRepository>) -> Self {
        Self { repository }
    }

    /// Create a new payment format
    pub async fn create(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        format_type: &str,
        payment_method: &str,
        file_template: Option<&str>,
        requires_bank_details: bool,
        supports_remittance: bool,
        supports_void: bool,
        max_payments_per_file: Option<i32>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentFormat> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if !VALID_FORMAT_TYPES.contains(&format_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid format type '{}'. Must be one of: {}", format_type, VALID_FORMAT_TYPES.join(", ")
            )));
        }
        if !VALID_PAYMENT_METHODS.contains(&payment_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment method '{}'. Must be one of: {}", payment_method, VALID_PAYMENT_METHODS.join(", ")
            )));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if let Some(max) = max_payments_per_file {
            if max <= 0 {
                return Err(AtlasError::ValidationFailed("Max payments per file must be positive".into()));
            }
        }

        // Check for duplicate code
        if self.repository.get_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Payment format '{}' already exists", code)));
        }

        info!("Creating payment format '{}' for org {}", code, org_id);

        self.repository.create(
            org_id, code, name, description,
            format_type, payment_method, file_template,
            requires_bank_details, supports_remittance, supports_void,
            max_payments_per_file, currency_code, created_by,
        ).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<PaymentFormat>> {
        self.repository.get(id).await
    }

    /// Get by code
    pub async fn get_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentFormat>> {
        self.repository.get_by_code(org_id, code).await
    }

    /// List payment formats
    pub async fn list(&self, org_id: Uuid, format_type: Option<&str>, payment_method: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<PaymentFormat>> {
        self.repository.list(org_id, format_type, payment_method, is_active).await
    }

    /// Update a payment format
    pub async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>, file_template: Option<&str>, max_payments: Option<i32>) -> AtlasResult<PaymentFormat> {
        let pf = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment format {} not found", id)))?;

        if !pf.is_active {
            return Err(AtlasError::ValidationFailed("Cannot update inactive payment format".into()));
        }

        if let Some(max) = max_payments {
            if max <= 0 {
                return Err(AtlasError::ValidationFailed("Max payments per file must be positive".into()));
            }
        }

        info!("Updating payment format {}", pf.code);
        self.repository.update(id, name, description, file_template, max_payments).await
    }

    /// Deactivate a payment format
    pub async fn deactivate(&self, id: Uuid) -> AtlasResult<PaymentFormat> {
        let pf = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment format {} not found", id)))?;

        if !pf.is_active {
            return Err(AtlasError::ValidationFailed("Payment format is already inactive".into()));
        }
        info!("Deactivating payment format {}", pf.code);
        self.repository.deactivate(id).await
    }

    /// Activate a payment format
    pub async fn activate(&self, id: Uuid) -> AtlasResult<PaymentFormat> {
        let pf = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment format {} not found", id)))?;

        if pf.is_active {
            return Err(AtlasError::ValidationFailed("Payment format is already active".into()));
        }
        info!("Activating payment format {}", pf.code);
        self.repository.activate(id).await
    }

    /// Delete a payment format (soft delete)
    pub async fn delete(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let _pf = self.repository.get_by_code(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment format '{}' not found", code)))?;

        info!("Deleting payment format {}", code);
        self.repository.delete(org_id, code).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentFormatDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        formats: std::sync::Mutex<Vec<PaymentFormat>>,
    }
    impl MockRepo { fn new() -> Self { Self { formats: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl PaymentFormatRepository for MockRepo {
        async fn create(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, format_type: &str, payment_method: &str, file_template: Option<&str>, requires_bank_details: bool, supports_remittance: bool, supports_void: bool, max_payments_per_file: Option<i32>, currency_code: &str, created_by: Option<Uuid>) -> AtlasResult<PaymentFormat> {
            let pf = PaymentFormat {
                id: Uuid::new_v4(), organization_id: org_id, code: code.into(), name: name.into(),
                description: description.map(Into::into), format_type: format_type.into(),
                payment_method: payment_method.into(), file_template: file_template.map(Into::into),
                requires_bank_details, supports_remittance, supports_void,
                max_payments_per_file, currency_code: currency_code.into(),
                is_active: true, metadata: serde_json::json!({}),
                created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.formats.lock().unwrap().push(pf.clone());
            Ok(pf)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<PaymentFormat>> {
            Ok(self.formats.lock().unwrap().iter().find(|f| f.id == id).cloned())
        }
        async fn get_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentFormat>> {
            Ok(self.formats.lock().unwrap().iter().find(|f| f.organization_id == org_id && f.code == code).cloned())
        }
        async fn list(&self, org_id: Uuid, format_type: Option<&str>, payment_method: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<PaymentFormat>> {
            Ok(self.formats.lock().unwrap().iter()
                .filter(|f| f.organization_id == org_id
                    && (format_type.is_none() || f.format_type == format_type.unwrap())
                    && (payment_method.is_none() || f.payment_method == payment_method.unwrap())
                    && (is_active.is_none() || f.is_active == is_active.unwrap()))
                .cloned().collect())
        }
        async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>, file_template: Option<&str>, max_payments: Option<i32>) -> AtlasResult<PaymentFormat> {
            let mut all = self.formats.lock().unwrap();
            let pf = all.iter_mut().find(|f| f.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            if let Some(n) = name { pf.name = n.into(); }
            if let Some(d) = description { pf.description = Some(d.into()); }
            if let Some(t) = file_template { pf.file_template = Some(t.into()); }
            if let Some(m) = max_payments { pf.max_payments_per_file = Some(m); }
            pf.updated_at = Utc::now();
            Ok(pf.clone())
        }
        async fn deactivate(&self, id: Uuid) -> AtlasResult<PaymentFormat> {
            let mut all = self.formats.lock().unwrap();
            let pf = all.iter_mut().find(|f| f.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            pf.is_active = false; pf.updated_at = Utc::now(); Ok(pf.clone())
        }
        async fn activate(&self, id: Uuid) -> AtlasResult<PaymentFormat> {
            let mut all = self.formats.lock().unwrap();
            let pf = all.iter_mut().find(|f| f.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            pf.is_active = true; pf.updated_at = Utc::now(); Ok(pf.clone())
        }
        async fn delete(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
            self.formats.lock().unwrap().retain(|f| !(f.organization_id == org_id && f.code == code));
            Ok(())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentFormatDashboard> {
            let all = self.formats.lock().unwrap();
            let org_fmts: Vec<_> = all.iter().filter(|f| f.organization_id == org_id).collect();
            Ok(PaymentFormatDashboard {
                organization_id: org_id,
                total_formats: org_fmts.len() as i32,
                active_formats: org_fmts.iter().filter(|f| f.is_active).count() as i32,
                by_format_type: serde_json::json!([]),
                by_payment_method: serde_json::json!([]),
            })
        }
    }

    fn eng() -> PaymentFormatEngine { PaymentFormatEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_format_types() {
        assert!(VALID_FORMAT_TYPES.contains(&"file"));
        assert!(VALID_FORMAT_TYPES.contains(&"printed_check"));
        assert!(VALID_FORMAT_TYPES.contains(&"xml"));
        assert!(VALID_FORMAT_TYPES.contains(&"json"));
        assert!(VALID_FORMAT_TYPES.contains(&"swift"));
    }

    #[test]
    fn test_valid_payment_methods() {
        assert!(VALID_PAYMENT_METHODS.contains(&"check"));
        assert!(VALID_PAYMENT_METHODS.contains(&"eft"));
        assert!(VALID_PAYMENT_METHODS.contains(&"wire"));
        assert!(VALID_PAYMENT_METHODS.contains(&"ach"));
        assert!(VALID_PAYMENT_METHODS.contains(&"sepa"));
    }

    #[tokio::test]
    async fn test_create_valid() {
        let pf = eng().create(
            Uuid::new_v4(), "EFT_US", "US EFT Format", Some("Standard US EFT"),
            "file", "eft", Some("eft_us_template.dat"), true, true, true, Some(10000), "USD", None,
        ).await.unwrap();
        assert_eq!(pf.code, "EFT_US");
        assert!(pf.is_active);
        assert_eq!(pf.format_type, "file");
    }

    #[tokio::test]
    async fn test_create_empty_code() {
        assert!(eng().create(
            Uuid::new_v4(), "", "Name", None, "file", "eft", None, false, true, true, None, "USD", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_empty_name() {
        assert!(eng().create(
            Uuid::new_v4(), "CODE", "", None, "file", "eft", None, false, true, true, None, "USD", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_format_type() {
        assert!(eng().create(
            Uuid::new_v4(), "CODE", "Name", None, "invalid", "eft", None, false, true, true, None, "USD", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_payment_method() {
        assert!(eng().create(
            Uuid::new_v4(), "CODE", "Name", None, "file", "invalid", None, false, true, true, None, "USD", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_currency() {
        assert!(eng().create(
            Uuid::new_v4(), "CODE", "Name", None, "file", "eft", None, false, true, true, None, "US", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_max_payments() {
        assert!(eng().create(
            Uuid::new_v4(), "CODE", "Name", None, "file", "eft", None, false, true, true, Some(-1), "USD", None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_duplicate_code() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create(org, "DUP", "First", None, "file", "eft", None, false, true, true, None, "USD", None).await.unwrap();
        assert!(e.create(org, "DUP", "Second", None, "file", "eft", None, false, true, true, None, "USD", None).await.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_activate() {
        let e = eng();
        let org = Uuid::new_v4();
        let pf = e.create(org, "TEST", "Test", None, "xml", "ach", None, false, true, true, None, "USD", None).await.unwrap();
        let deactivated = e.deactivate(pf.id).await.unwrap();
        assert!(!deactivated.is_active);
        let activated = e.activate(pf.id).await.unwrap();
        assert!(activated.is_active);
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive() {
        let e = eng();
        let org = Uuid::new_v4();
        let pf = e.create(org, "TEST", "Test", None, "xml", "ach", None, false, true, true, None, "USD", None).await.unwrap();
        let _ = e.deactivate(pf.id).await.unwrap();
        assert!(e.deactivate(pf.id).await.is_err());
    }

    #[tokio::test]
    async fn test_activate_already_active() {
        let e = eng();
        let org = Uuid::new_v4();
        let pf = e.create(org, "TEST", "Test", None, "xml", "ach", None, false, true, true, None, "USD", None).await.unwrap();
        assert!(e.activate(pf.id).await.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let e = eng();
        let org = Uuid::new_v4();
        let pf = e.create(org, "TEST", "Test", None, "xml", "ach", None, false, true, true, None, "USD", None).await.unwrap();
        let updated = e.update(pf.id, Some("Updated Name"), Some("New desc"), Some("template.xml"), Some(5000)).await.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.max_payments_per_file.unwrap(), 5000);
    }

    #[tokio::test]
    async fn test_update_negative_max() {
        let e = eng();
        let org = Uuid::new_v4();
        let pf = e.create(org, "TEST", "Test", None, "xml", "ach", None, false, true, true, None, "USD", None).await.unwrap();
        assert!(e.update(pf.id, None, None, None, Some(-5)).await.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let e = eng();
        let org = Uuid::new_v4();
        let pf = e.create(org, "DEL", "To Delete", None, "file", "eft", None, false, true, true, None, "USD", None).await.unwrap();
        e.delete(org, "DEL").await.unwrap();
        assert!(e.get_by_code(org, "DEL").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create(org, "F1", "Format 1", None, "file", "eft", None, false, true, true, None, "USD", None).await.unwrap();
        let _ = e.create(org, "F2", "Format 2", None, "xml", "ach", None, false, true, true, None, "USD", None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_formats, 2);
        assert_eq!(dash.active_formats, 2);
    }
}
