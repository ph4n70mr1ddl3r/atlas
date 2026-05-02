//! Prepayment Application Module
//!
//! Oracle Fusion Cloud ERP-inspired Prepayment Application.
//! Applies prepayment invoices (advances) to standard invoices,
//! netting the amounts against supplier payables.
//!
//! Oracle Fusion equivalent: Financials > Payables > Prepayment Application

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
pub struct PrepaymentApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_number: String,
    pub prepayment_invoice_id: Uuid,
    pub prepayment_invoice_number: Option<String>,
    pub standard_invoice_id: Uuid,
    pub standard_invoice_number: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub applied_amount: String,
    pub remaining_prepayment_amount: String,
    pub currency_code: String,
    pub application_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub status: String,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepaymentDashboard {
    pub organization_id: Uuid,
    pub total_applications: i32,
    pub draft_applications: i32,
    pub applied_applications: i32,
    pub cancelled_applications: i32,
    pub total_applied_amount: String,
    pub by_supplier: serde_json::Value,
}

// ============================================================================
// Constants
// ============================================================================

const VALID_STATUSES: &[&str] = &["draft", "applied", "cancelled", "reversed"];

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait PrepaymentApplicationRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, application_number: &str,
        prepayment_invoice_id: Uuid, prepayment_invoice_number: Option<&str>,
        standard_invoice_id: Uuid, standard_invoice_number: Option<&str>,
        supplier_id: Uuid, supplier_number: Option<&str>,
        applied_amount: &str, remaining_prepayment_amount: &str,
        currency_code: &str, application_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate, status: &str,
        reason: Option<&str>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PrepaymentApplication>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<PrepaymentApplication>>;
    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<PrepaymentApplication>>;
    async fn list(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<PrepaymentApplication>>;
    async fn update_status(&self, id: Uuid, status: &str) -> AtlasResult<PrepaymentApplication>;
    async fn update_remaining(&self, id: Uuid, remaining: &str) -> AtlasResult<PrepaymentApplication>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PrepaymentDashboard>;
}

/// PostgreSQL stub implementation
pub struct PostgresPrepaymentApplicationRepository { pool: PgPool }
impl PostgresPrepaymentApplicationRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl PrepaymentApplicationRepository for PostgresPrepaymentApplicationRepository {
    async fn create(&self, _: Uuid, _: &str, _: Uuid, _: Option<&str>, _: Uuid, _: Option<&str>, _: Uuid, _: Option<&str>, _: &str, _: &str, _: &str, _: chrono::NaiveDate, _: chrono::NaiveDate, _: &str, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<PrepaymentApplication> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<PrepaymentApplication>> { Ok(None) }
    async fn get_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<PrepaymentApplication>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<PrepaymentApplication>> { Ok(vec![]) }
    async fn update_status(&self, _: Uuid, _: &str) -> AtlasResult<PrepaymentApplication> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_remaining(&self, _: Uuid, _: &str) -> AtlasResult<PrepaymentApplication> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PrepaymentDashboard> {
        Ok(PrepaymentDashboard {
            organization_id: org_id, total_applications: 0, draft_applications: 0,
            applied_applications: 0, cancelled_applications: 0,
            total_applied_amount: "0".into(), by_supplier: serde_json::json!([]),
        })
    }
}

// ============================================================================
// Engine
// ============================================================================

use std::sync::Arc;
use tracing::info;

pub struct PrepaymentApplicationEngine {
    repository: Arc<dyn PrepaymentApplicationRepository>,
}

impl PrepaymentApplicationEngine {
    pub fn new(repository: Arc<dyn PrepaymentApplicationRepository>) -> Self {
        Self { repository }
    }

    /// Apply a prepayment to a standard invoice
    pub async fn apply(
        &self,
        org_id: Uuid,
        prepayment_invoice_id: Uuid,
        prepayment_invoice_number: Option<&str>,
        standard_invoice_id: Uuid,
        standard_invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        applied_amount: &str,
        remaining_prepayment_amount: &str,
        currency_code: &str,
        application_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        reason: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PrepaymentApplication> {
        // Validate amounts
        let applied: f64 = applied_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid applied amount".into()))?;
        if applied <= 0.0 {
            return Err(AtlasError::ValidationFailed("Applied amount must be positive".into()));
        }

        let remaining: f64 = remaining_prepayment_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid remaining prepayment amount".into()))?;
        if remaining < 0.0 {
            return Err(AtlasError::ValidationFailed("Remaining prepayment amount cannot be negative".into()));
        }

        // Applied amount cannot exceed remaining
        if applied > remaining {
            return Err(AtlasError::ValidationFailed(
                "Applied amount cannot exceed remaining prepayment amount".into()
            ));
        }

        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        if gl_date < application_date {
            return Err(AtlasError::ValidationFailed("GL date cannot be before application date".into()));
        }

        // Same supplier check
        // In production, we'd verify both invoices belong to the same supplier

        let number = format!("PA-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating prepayment application {} for supplier {}", number, supplier_id);

        self.repository.create(
            org_id, &number,
            prepayment_invoice_id, prepayment_invoice_number,
            standard_invoice_id, standard_invoice_number,
            supplier_id, supplier_number,
            applied_amount, remaining_prepayment_amount,
            currency_code, application_date, gl_date,
            "draft", reason, notes, created_by,
        ).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<PrepaymentApplication>> {
        self.repository.get(id).await
    }

    /// Get by number
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<PrepaymentApplication>> {
        self.repository.get_by_number(org_id, number).await
    }

    /// List applications
    pub async fn list(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<PrepaymentApplication>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list(org_id, status, supplier_id).await
    }

    /// Confirm/apply a draft application (posts to GL)
    pub async fn confirm(&self, id: Uuid) -> AtlasResult<PrepaymentApplication> {
        let pa = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Prepayment application {} not found", id)))?;

        if pa.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm application in '{}' status. Must be 'draft'.", pa.status
            )));
        }
        info!("Confirming prepayment application {}", pa.application_number);
        self.repository.update_status(id, "applied").await
    }

    /// Cancel a draft application
    pub async fn cancel(&self, id: Uuid) -> AtlasResult<PrepaymentApplication> {
        let pa = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Prepayment application {} not found", id)))?;

        if pa.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel application in '{}' status. Must be 'draft'.", pa.status
            )));
        }
        info!("Cancelling prepayment application {}", pa.application_number);
        self.repository.update_status(id, "cancelled").await
    }

    /// Reverse an applied application
    pub async fn reverse(&self, id: Uuid) -> AtlasResult<PrepaymentApplication> {
        let pa = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Prepayment application {} not found", id)))?;

        if pa.status != "applied" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse application in '{}' status. Must be 'applied'.", pa.status
            )));
        }
        info!("Reversing prepayment application {}", pa.application_number);
        self.repository.update_status(id, "reversed").await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PrepaymentDashboard> {
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
        items: std::sync::Mutex<Vec<PrepaymentApplication>>,
    }
    impl MockRepo { fn new() -> Self { Self { items: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl PrepaymentApplicationRepository for MockRepo {
        async fn create(&self, org_id: Uuid, number: &str, prepay_id: Uuid, prepay_num: Option<&str>, std_id: Uuid, std_num: Option<&str>, supplier_id: Uuid, supplier_num: Option<&str>, applied: &str, remaining: &str, currency: &str, app_date: chrono::NaiveDate, gl_date: chrono::NaiveDate, status: &str, reason: Option<&str>, notes: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<PrepaymentApplication> {
            let pa = PrepaymentApplication {
                id: Uuid::new_v4(), organization_id: org_id, application_number: number.into(),
                prepayment_invoice_id: prepay_id, prepayment_invoice_number: prepay_num.map(Into::into),
                standard_invoice_id: std_id, standard_invoice_number: std_num.map(Into::into),
                supplier_id, supplier_number: supplier_num.map(Into::into),
                applied_amount: applied.into(), remaining_prepayment_amount: remaining.into(),
                currency_code: currency.into(), application_date: app_date, gl_date: gl_date,
                status: status.into(), reason: reason.map(Into::into), notes: notes.map(Into::into),
                metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.items.lock().unwrap().push(pa.clone());
            Ok(pa)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<PrepaymentApplication>> {
            Ok(self.items.lock().unwrap().iter().find(|p| p.id == id).cloned())
        }
        async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<PrepaymentApplication>> {
            Ok(self.items.lock().unwrap().iter().find(|p| p.organization_id == org_id && p.application_number == number).cloned())
        }
        async fn list(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<PrepaymentApplication>> {
            Ok(self.items.lock().unwrap().iter()
                .filter(|p| p.organization_id == org_id
                    && (status.is_none() || p.status == status.unwrap())
                    && (supplier_id.is_none() || p.supplier_id == supplier_id.unwrap()))
                .cloned().collect())
        }
        async fn update_status(&self, id: Uuid, status: &str) -> AtlasResult<PrepaymentApplication> {
            let mut all = self.items.lock().unwrap();
            let pa = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            pa.status = status.into(); pa.updated_at = Utc::now(); Ok(pa.clone())
        }
        async fn update_remaining(&self, id: Uuid, remaining: &str) -> AtlasResult<PrepaymentApplication> {
            let mut all = self.items.lock().unwrap();
            let pa = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            pa.remaining_prepayment_amount = remaining.into(); pa.updated_at = Utc::now(); Ok(pa.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PrepaymentDashboard> {
            let all = self.items.lock().unwrap();
            let org_items: Vec<_> = all.iter().filter(|p| p.organization_id == org_id).collect();
            Ok(PrepaymentDashboard {
                organization_id: org_id,
                total_applications: org_items.len() as i32,
                draft_applications: org_items.iter().filter(|p| p.status == "draft").count() as i32,
                applied_applications: org_items.iter().filter(|p| p.status == "applied").count() as i32,
                cancelled_applications: org_items.iter().filter(|p| p.status == "cancelled").count() as i32,
                total_applied_amount: org_items.iter().filter(|p| p.status == "applied").map(|p| p.applied_amount.parse::<f64>().unwrap_or(0.0)).sum::<f64>().to_string(),
                by_supplier: serde_json::json!([]),
            })
        }
    }

    fn eng() -> PrepaymentApplicationEngine { PrepaymentApplicationEngine::new(Arc::new(MockRepo::new())) }

    fn today() -> chrono::NaiveDate { chrono::Utc::now().date_naive() }
    fn tomorrow() -> chrono::NaiveDate { today() + chrono::Duration::days(1) }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"applied"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
        assert!(VALID_STATUSES.contains(&"reversed"));
    }

    #[tokio::test]
    async fn test_apply_valid() {
        let pa = eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), Some("PREPAY-001"),
            Uuid::new_v4(), Some("INV-001"),
            Uuid::new_v4(), Some("SUP-001"),
            "5000.00", "10000.00", "USD",
            today(), today(), Some("Apply prepayment"), None, None,
        ).await.unwrap();
        assert_eq!(pa.status, "draft");
        assert_eq!(pa.applied_amount, "5000.00");
    }

    #[tokio::test]
    async fn test_apply_zero_amount() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "0", "10000", "USD", today(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_apply_negative_amount() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "-500", "10000", "USD", today(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_apply_invalid_amount() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "abc", "10000", "USD", today(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_apply_exceeds_remaining() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "15000", "10000", "USD", today(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_apply_negative_remaining() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "500", "-100", "USD", today(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_apply_invalid_currency() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "500", "10000", "US", today(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_apply_gl_date_before_app_date() {
        assert!(eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "500", "10000", "USD", tomorrow(), today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_confirm() {
        let e = eng();
        let org = Uuid::new_v4();
        let pa = e.apply(org, Uuid::new_v4(), None, Uuid::new_v4(), None, Uuid::new_v4(), None, "500", "10000", "USD", today(), today(), None, None, None).await.unwrap();
        let confirmed = e.confirm(pa.id).await.unwrap();
        assert_eq!(confirmed.status, "applied");
    }

    #[tokio::test]
    async fn test_confirm_not_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let pa = e.apply(org, Uuid::new_v4(), None, Uuid::new_v4(), None, Uuid::new_v4(), None, "500", "10000", "USD", today(), today(), None, None, None).await.unwrap();
        let _ = e.confirm(pa.id).await.unwrap();
        assert!(e.confirm(pa.id).await.is_err()); // already applied
    }

    #[tokio::test]
    async fn test_cancel() {
        let e = eng();
        let org = Uuid::new_v4();
        let pa = e.apply(org, Uuid::new_v4(), None, Uuid::new_v4(), None, Uuid::new_v4(), None, "500", "10000", "USD", today(), today(), None, None, None).await.unwrap();
        let cancelled = e.cancel(pa.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_not_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let pa = e.apply(org, Uuid::new_v4(), None, Uuid::new_v4(), None, Uuid::new_v4(), None, "500", "10000", "USD", today(), today(), None, None, None).await.unwrap();
        let _ = e.confirm(pa.id).await.unwrap();
        assert!(e.cancel(pa.id).await.is_err());
    }

    #[tokio::test]
    async fn test_reverse() {
        let e = eng();
        let org = Uuid::new_v4();
        let pa = e.apply(org, Uuid::new_v4(), None, Uuid::new_v4(), None, Uuid::new_v4(), None, "500", "10000", "USD", today(), today(), None, None, None).await.unwrap();
        let _ = e.confirm(pa.id).await.unwrap();
        let reversed = e.reverse(pa.id).await.unwrap();
        assert_eq!(reversed.status, "reversed");
    }

    #[tokio::test]
    async fn test_reverse_not_applied() {
        let e = eng();
        let org = Uuid::new_v4();
        let pa = e.apply(org, Uuid::new_v4(), None, Uuid::new_v4(), None, Uuid::new_v4(), None, "500", "10000", "USD", today(), today(), None, None, None).await.unwrap();
        assert!(e.reverse(pa.id).await.is_err()); // draft
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let supplier = Uuid::new_v4();

        let pa = e.apply(
            org, Uuid::new_v4(), Some("PRE-001"),
            Uuid::new_v4(), Some("INV-100"),
            supplier, Some("ACME"),
            "7500.00", "15000.00", "USD",
            today(), today(), Some("Partial application"), None, None,
        ).await.unwrap();
        assert_eq!(pa.status, "draft");

        let confirmed = e.confirm(pa.id).await.unwrap();
        assert_eq!(confirmed.status, "applied");

        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_applications, 1);
        assert_eq!(dash.applied_applications, 1);
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_valid() {
        assert!(eng().list(Uuid::new_v4(), Some("draft"), None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_apply_exact_remaining() {
        let pa = eng().apply(
            Uuid::new_v4(), Uuid::new_v4(), None,
            Uuid::new_v4(), None, Uuid::new_v4(), None,
            "10000.00", "10000.00", "USD", today(), today(), None, None, None,
        ).await.unwrap();
        assert_eq!(pa.applied_amount, "10000.00");
    }
}
