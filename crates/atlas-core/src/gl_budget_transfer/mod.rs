//! GL Budget Transfer Module
//!
//! Oracle Fusion Cloud ERP-inspired General Ledger Budget Transfers.
//! Enables transferring budget amounts between accounts, periods, departments,
//! and cost centers with approval workflow and audit trail.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Budget Transfers

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
pub struct GlBudgetTransfer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transfer_number: String,
    pub description: Option<String>,
    pub transfer_date: chrono::NaiveDate,
    pub effective_date: chrono::NaiveDate,
    pub budget_name: Option<String>,
    pub transfer_type: String,
    pub from_account_combination: Option<String>,
    pub from_department: Option<String>,
    pub from_period: Option<String>,
    pub to_account_combination: Option<String>,
    pub to_department: Option<String>,
    pub to_period: Option<String>,
    pub transfer_amount: String,
    pub currency_code: String,
    pub reason: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTransferDashboard {
    pub organization_id: Uuid,
    pub total_transfers: i32,
    pub draft_transfers: i32,
    pub pending_transfers: i32,
    pub approved_transfers: i32,
    pub rejected_transfers: i32,
    pub completed_transfers: i32,
    pub total_transfer_amount: String,
    pub by_transfer_type: serde_json::Value,
}

// ============================================================================
// Constants
// ============================================================================

const VALID_STATUSES: &[&str] = &["draft", "pending_approval", "approved", "rejected", "completed", "cancelled"];

const VALID_TRANSFER_TYPES: &[&str] = &[
    "account_to_account", "period_to_period", "department_to_department",
    "cost_center_transfer", "budget_reallocation",
];

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait GlBudgetTransferRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, transfer_number: &str, description: Option<&str>,
        transfer_date: chrono::NaiveDate, effective_date: chrono::NaiveDate,
        budget_name: Option<&str>, transfer_type: &str,
        from_account_combination: Option<&str>, from_department: Option<&str>, from_period: Option<&str>,
        to_account_combination: Option<&str>, to_department: Option<&str>, to_period: Option<&str>,
        transfer_amount: &str, currency_code: &str, reason: Option<&str>,
        status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<GlBudgetTransfer>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<GlBudgetTransfer>>;
    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GlBudgetTransfer>>;
    async fn list(&self, org_id: Uuid, status: Option<&str>, transfer_type: Option<&str>) -> AtlasResult<Vec<GlBudgetTransfer>>;
    async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<GlBudgetTransfer>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetTransferDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresGlBudgetTransferRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresGlBudgetTransferRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl GlBudgetTransferRepository for PostgresGlBudgetTransferRepository {
    async fn create(&self, _: Uuid, _: &str, _: Option<&str>, _: chrono::NaiveDate, _: chrono::NaiveDate, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<Uuid>) -> AtlasResult<GlBudgetTransfer> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<GlBudgetTransfer>> { Ok(None) }
    async fn get_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<GlBudgetTransfer>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<GlBudgetTransfer>> { Ok(vec![]) }
    async fn update_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<GlBudgetTransfer> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetTransferDashboard> {
        Ok(BudgetTransferDashboard {
            organization_id: org_id, total_transfers: 0, draft_transfers: 0,
            pending_transfers: 0, approved_transfers: 0, rejected_transfers: 0,
            completed_transfers: 0, total_transfer_amount: "0".into(), by_transfer_type: serde_json::json!([]),
        })
    }
}

// ============================================================================
// Engine
// ============================================================================

use std::sync::Arc;
use tracing::info;

pub struct GlBudgetTransferEngine {
    repository: Arc<dyn GlBudgetTransferRepository>,
}

impl GlBudgetTransferEngine {
    pub fn new(repository: Arc<dyn GlBudgetTransferRepository>) -> Self {
        Self { repository }
    }

    /// Create a new budget transfer
    pub async fn create(
        &self,
        org_id: Uuid,
        description: Option<&str>,
        transfer_date: chrono::NaiveDate,
        effective_date: chrono::NaiveDate,
        budget_name: Option<&str>,
        transfer_type: &str,
        from_account_combination: Option<&str>,
        from_department: Option<&str>,
        from_period: Option<&str>,
        to_account_combination: Option<&str>,
        to_department: Option<&str>,
        to_period: Option<&str>,
        transfer_amount: &str,
        currency_code: &str,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlBudgetTransfer> {
        if !VALID_TRANSFER_TYPES.contains(&transfer_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transfer type '{}'. Must be one of: {}", transfer_type, VALID_TRANSFER_TYPES.join(", ")
            )));
        }

        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        let amount: f64 = transfer_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid transfer amount".into()))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("Transfer amount must be positive".into()));
        }

        if effective_date < transfer_date {
            return Err(AtlasError::ValidationFailed("Effective date cannot be before transfer date".into()));
        }

        let number = format!("BT-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating budget transfer {} for org {}", number, org_id);

        self.repository.create(
            org_id, &number, description,
            transfer_date, effective_date, budget_name, transfer_type,
            from_account_combination, from_department, from_period,
            to_account_combination, to_department, to_period,
            transfer_amount, currency_code, reason,
            "draft", created_by,
        ).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<GlBudgetTransfer>> {
        self.repository.get(id).await
    }

    /// Get by number
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GlBudgetTransfer>> {
        self.repository.get_by_number(org_id, number).await
    }

    /// List budget transfers
    pub async fn list(&self, org_id: Uuid, status: Option<&str>, transfer_type: Option<&str>) -> AtlasResult<Vec<GlBudgetTransfer>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list(org_id, status, transfer_type).await
    }

    /// Submit a draft transfer for approval
    pub async fn submit(&self, id: Uuid) -> AtlasResult<GlBudgetTransfer> {
        let bt = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Budget transfer {} not found", id)))?;

        if bt.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit transfer in '{}' status. Must be 'draft'.", bt.status
            )));
        }
        info!("Submitting budget transfer {} for approval", bt.transfer_number);
        self.repository.update_status(id, "pending_approval", None).await
    }

    /// Approve a pending transfer
    pub async fn approve(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<GlBudgetTransfer> {
        let bt = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Budget transfer {} not found", id)))?;

        if bt.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve transfer in '{}' status. Must be 'pending_approval'.", bt.status
            )));
        }
        info!("Approving budget transfer {}", bt.transfer_number);
        self.repository.update_status(id, "approved", Some(approved_by)).await
    }

    /// Complete an approved transfer (post to GL)
    pub async fn complete(&self, id: Uuid) -> AtlasResult<GlBudgetTransfer> {
        let bt = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Budget transfer {} not found", id)))?;

        if bt.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete transfer in '{}' status. Must be 'approved'.", bt.status
            )));
        }
        info!("Completing budget transfer {}", bt.transfer_number);
        self.repository.update_status(id, "completed", None).await
    }

    /// Reject a pending transfer
    pub async fn reject(&self, id: Uuid) -> AtlasResult<GlBudgetTransfer> {
        let bt = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Budget transfer {} not found", id)))?;

        if bt.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject transfer in '{}' status. Must be 'pending_approval'.", bt.status
            )));
        }
        self.repository.update_status(id, "rejected", None).await
    }

    /// Cancel a draft transfer
    pub async fn cancel(&self, id: Uuid) -> AtlasResult<GlBudgetTransfer> {
        let bt = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Budget transfer {} not found", id)))?;

        if bt.status != "draft" && bt.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel transfer in '{}' status", bt.status
            )));
        }
        info!("Cancelling budget transfer {}", bt.transfer_number);
        self.repository.update_status(id, "cancelled", None).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetTransferDashboard> {
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
        items: std::sync::Mutex<Vec<GlBudgetTransfer>>,
    }
    impl MockRepo { fn new() -> Self { Self { items: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl GlBudgetTransferRepository for MockRepo {
        async fn create(&self, org_id: Uuid, number: &str, description: Option<&str>, transfer_date: chrono::NaiveDate, effective_date: chrono::NaiveDate, budget_name: Option<&str>, transfer_type: &str, from_account_combination: Option<&str>, from_department: Option<&str>, from_period: Option<&str>, to_account_combination: Option<&str>, to_department: Option<&str>, to_period: Option<&str>, transfer_amount: &str, currency_code: &str, reason: Option<&str>, status: &str, created_by: Option<Uuid>) -> AtlasResult<GlBudgetTransfer> {
            let bt = GlBudgetTransfer {
                id: Uuid::new_v4(), organization_id: org_id, transfer_number: number.into(),
                description: description.map(Into::into), transfer_date, effective_date,
                budget_name: budget_name.map(Into::into), transfer_type: transfer_type.into(),
                from_account_combination: from_account_combination.map(Into::into),
                from_department: from_department.map(Into::into), from_period: from_period.map(Into::into),
                to_account_combination: to_account_combination.map(Into::into),
                to_department: to_department.map(Into::into), to_period: to_period.map(Into::into),
                transfer_amount: transfer_amount.into(), currency_code: currency_code.into(),
                reason: reason.map(Into::into), approved_by: None, approved_at: None,
                status: status.into(), metadata: serde_json::json!({}),
                created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.items.lock().unwrap().push(bt.clone());
            Ok(bt)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<GlBudgetTransfer>> {
            Ok(self.items.lock().unwrap().iter().find(|b| b.id == id).cloned())
        }
        async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GlBudgetTransfer>> {
            Ok(self.items.lock().unwrap().iter().find(|b| b.organization_id == org_id && b.transfer_number == number).cloned())
        }
        async fn list(&self, org_id: Uuid, status: Option<&str>, _tt: Option<&str>) -> AtlasResult<Vec<GlBudgetTransfer>> {
            Ok(self.items.lock().unwrap().iter()
                .filter(|b| b.organization_id == org_id && (status.is_none() || b.status == status.unwrap()))
                .cloned().collect())
        }
        async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<GlBudgetTransfer> {
            let mut all = self.items.lock().unwrap();
            let bt = all.iter_mut().find(|b| b.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            bt.status = status.into();
            if let Some(ab) = approved_by { bt.approved_by = Some(ab); bt.approved_at = Some(Utc::now()); }
            bt.updated_at = Utc::now();
            Ok(bt.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetTransferDashboard> {
            let all = self.items.lock().unwrap();
            let org_items: Vec<_> = all.iter().filter(|b| b.organization_id == org_id).collect();
            Ok(BudgetTransferDashboard {
                organization_id: org_id,
                total_transfers: org_items.len() as i32,
                draft_transfers: org_items.iter().filter(|b| b.status == "draft").count() as i32,
                pending_transfers: org_items.iter().filter(|b| b.status == "pending_approval").count() as i32,
                approved_transfers: org_items.iter().filter(|b| b.status == "approved").count() as i32,
                rejected_transfers: org_items.iter().filter(|b| b.status == "rejected").count() as i32,
                completed_transfers: org_items.iter().filter(|b| b.status == "completed").count() as i32,
                total_transfer_amount: org_items.iter().map(|b| b.transfer_amount.parse::<f64>().unwrap_or(0.0)).sum::<f64>().to_string(),
                by_transfer_type: serde_json::json!([]),
            })
        }
    }

    fn eng() -> GlBudgetTransferEngine { GlBudgetTransferEngine::new(Arc::new(MockRepo::new())) }

    fn today() -> chrono::NaiveDate { chrono::Utc::now().date_naive() }
    fn tomorrow() -> chrono::NaiveDate { today() + chrono::Duration::days(1) }

    #[test]
    fn test_valid_transfer_types() {
        assert!(VALID_TRANSFER_TYPES.contains(&"account_to_account"));
        assert!(VALID_TRANSFER_TYPES.contains(&"period_to_period"));
        assert!(VALID_TRANSFER_TYPES.contains(&"department_to_department"));
        assert!(VALID_TRANSFER_TYPES.contains(&"budget_reallocation"));
    }

    #[tokio::test]
    async fn test_create_valid() {
        let bt = eng().create(
            Uuid::new_v4(), Some("Transfer budget"), today(), tomorrow(),
            Some("FY2025"), "account_to_account",
            Some("1000.100.001"), Some("IT"), Some("JAN-25"),
            Some("2000.200.002"), Some("Finance"), Some("FEB-25"),
            "50000.00", "USD", Some("Reallocate surplus"), None,
        ).await.unwrap();
        assert_eq!(bt.status, "draft");
        assert_eq!(bt.transfer_amount, "50000.00");
    }

    #[tokio::test]
    async fn test_create_invalid_type() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), None, "invalid",
            None, None, None, None, None, None, "100", "USD", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_currency() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "100", "US", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_zero_amount() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "0", "USD", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_amount() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "-100", "USD", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_amount() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "abc", "USD", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_effective_before_transfer() {
        assert!(eng().create(
            Uuid::new_v4(), None, tomorrow(), today(), None, "account_to_account",
            None, None, None, None, None, None, "100", "USD", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_workflow_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        assert_eq!(bt.status, "draft");

        let submitted = e.submit(bt.id).await.unwrap();
        assert_eq!(submitted.status, "pending_approval");

        let approved = e.approve(bt.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");

        let completed = e.complete(bt.id).await.unwrap();
        assert_eq!(completed.status, "completed");
    }

    #[tokio::test]
    async fn test_submit_not_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        let _ = e.submit(bt.id).await.unwrap();
        assert!(e.submit(bt.id).await.is_err()); // already pending
    }

    #[tokio::test]
    async fn test_approve_not_pending() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        assert!(e.approve(bt.id, Uuid::new_v4()).await.is_err()); // draft, not pending
    }

    #[tokio::test]
    async fn test_complete_not_approved() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        assert!(e.complete(bt.id).await.is_err()); // draft
    }

    #[tokio::test]
    async fn test_reject() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        let _ = e.submit(bt.id).await.unwrap();
        let rejected = e.reject(bt.id).await.unwrap();
        assert_eq!(rejected.status, "rejected");
    }

    #[tokio::test]
    async fn test_cancel_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        let cancelled = e.cancel(bt.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_pending() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        let _ = e.submit(bt.id).await.unwrap();
        let cancelled = e.cancel(bt.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_approved_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let bt = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        let _ = e.submit(bt.id).await.unwrap();
        let _ = e.approve(bt.id, Uuid::new_v4()).await.unwrap();
        assert!(e.cancel(bt.id).await.is_err());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create(org, None, today(), tomorrow(), None, "account_to_account",
            None, None, None, None, None, None, "5000", "USD", None, None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_transfers, 1);
        assert_eq!(dash.draft_transfers, 1);
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }
}
