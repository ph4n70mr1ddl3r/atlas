//! Asset Reclassification Module
//!
//! Oracle Fusion Cloud ERP-inspired Fixed Asset Reclassification.
//! Allows changing asset category, depreciation method, account assignments,
//! and other attributes with approval workflow and GL impact.
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > Asset Reclassification

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
pub struct AssetReclassification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub reclassification_number: String,
    pub asset_id: Uuid,
    pub asset_number: Option<String>,
    pub asset_name: Option<String>,
    pub reclassification_type: String,
    pub reason: Option<String>,
    pub from_category_id: Option<Uuid>,
    pub from_category_code: Option<String>,
    pub from_asset_type: Option<String>,
    pub from_depreciation_method: Option<String>,
    pub from_useful_life_months: Option<i32>,
    pub from_asset_account_code: Option<String>,
    pub from_depr_expense_account_code: Option<String>,
    pub to_category_id: Option<Uuid>,
    pub to_category_code: Option<String>,
    pub to_asset_type: Option<String>,
    pub to_depreciation_method: Option<String>,
    pub to_useful_life_months: Option<i32>,
    pub to_asset_account_code: Option<String>,
    pub to_depr_expense_account_code: Option<String>,
    pub effective_date: chrono::NaiveDate,
    pub amortization_adjustment: Option<String>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclassificationDashboard {
    pub organization_id: Uuid,
    pub total_reclassifications: i32,
    pub pending_reclassifications: i32,
    pub approved_reclassifications: i32,
    pub completed_reclassifications: i32,
    pub rejected_reclassifications: i32,
    pub by_type: serde_json::Value,
}

// ============================================================================
// Constants
// ============================================================================

const VALID_STATUSES: &[&str] = &["pending", "approved", "completed", "rejected", "cancelled"];

const VALID_RECLASS_TYPES: &[&str] = &[
    "category_change", "depreciation_change", "account_change",
    "location_change", "type_change", "comprehensive",
];

const VALID_AMORTIZATION_ADJUSTMENTS: &[&str] = &[
    "catchup", "prospectively", "restatement",
];

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait AssetReclassificationRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, reclassification_number: &str,
        asset_id: Uuid, asset_number: Option<&str>, asset_name: Option<&str>,
        reclassification_type: &str, reason: Option<&str>,
        from_category_id: Option<Uuid>, from_category_code: Option<&str>,
        from_asset_type: Option<&str>, from_depreciation_method: Option<&str>,
        from_useful_life_months: Option<i32>,
        from_asset_account_code: Option<&str>, from_depr_expense_account_code: Option<&str>,
        to_category_id: Option<Uuid>, to_category_code: Option<&str>,
        to_asset_type: Option<&str>, to_depreciation_method: Option<&str>,
        to_useful_life_months: Option<i32>,
        to_asset_account_code: Option<&str>, to_depr_expense_account_code: Option<&str>,
        effective_date: chrono::NaiveDate, amortization_adjustment: Option<&str>,
        status: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<AssetReclassification>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetReclassification>>;
    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetReclassification>>;
    async fn list(&self, org_id: Uuid, status: Option<&str>, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetReclassification>>;
    async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<AssetReclassification>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ReclassificationDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresAssetReclassificationRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresAssetReclassificationRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl AssetReclassificationRepository for PostgresAssetReclassificationRepository {
    async fn create(&self, _: Uuid, _: &str, _: Uuid, _: Option<&str>, _: Option<&str>, _: &str, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<i32>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<i32>, _: Option<&str>, _: Option<&str>, _: chrono::NaiveDate, _: Option<&str>, _: &str, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<AssetReclassification> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<AssetReclassification>> { Ok(None) }
    async fn get_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<AssetReclassification>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<AssetReclassification>> { Ok(vec![]) }
    async fn update_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<AssetReclassification> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ReclassificationDashboard> {
        Ok(ReclassificationDashboard { organization_id: org_id, total_reclassifications: 0, pending_reclassifications: 0, approved_reclassifications: 0, completed_reclassifications: 0, rejected_reclassifications: 0, by_type: serde_json::json!([]) })
    }
}

// ============================================================================
// Engine
// ============================================================================

use std::sync::Arc;
use tracing::info;

pub struct AssetReclassificationEngine {
    repository: Arc<dyn AssetReclassificationRepository>,
}

impl AssetReclassificationEngine {
    pub fn new(repository: Arc<dyn AssetReclassificationRepository>) -> Self {
        Self { repository }
    }

    /// Create a new reclassification request
    pub async fn create(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        asset_number: Option<&str>,
        asset_name: Option<&str>,
        reclassification_type: &str,
        reason: Option<&str>,
        from_category_code: Option<&str>,
        from_asset_type: Option<&str>,
        from_depreciation_method: Option<&str>,
        from_useful_life_months: Option<i32>,
        _from_asset_account_code: Option<&str>,
        to_category_code: Option<&str>,
        to_asset_type: Option<&str>,
        to_depreciation_method: Option<&str>,
        to_useful_life_months: Option<i32>,
        to_asset_account_code: Option<&str>,
        effective_date: chrono::NaiveDate,
        amortization_adjustment: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetReclassification> {
        if !VALID_RECLASS_TYPES.contains(&reclassification_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid reclassification type '{}'. Must be one of: {}",
                reclassification_type, VALID_RECLASS_TYPES.join(", ")
            )));
        }

        if let Some(adj) = amortization_adjustment {
            if !VALID_AMORTIZATION_ADJUSTMENTS.contains(&adj) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid amortization adjustment '{}'. Must be one of: {}",
                    adj, VALID_AMORTIZATION_ADJUSTMENTS.join(", ")
                )));
            }
        }

        if let Some(months) = to_useful_life_months {
            if months <= 0 {
                return Err(AtlasError::ValidationFailed("Useful life months must be positive".into()));
            }
        }

        let number = format!("RECLASS-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating asset reclassification {} for asset {}", number, asset_id);

        self.repository.create(
            org_id, &number, asset_id, asset_number, asset_name,
            reclassification_type, reason,
            None, from_category_code,
            from_asset_type, from_depreciation_method, from_useful_life_months,
            None, None, // from depr expense account
            None, to_category_code,
            to_asset_type, to_depreciation_method, to_useful_life_months,
            to_asset_account_code, None, // to depr expense account
            effective_date, amortization_adjustment,
            "pending", notes, created_by,
        ).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetReclassification>> {
        self.repository.get(id).await
    }

    /// Get by number
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetReclassification>> {
        self.repository.get_by_number(org_id, number).await
    }

    /// List reclassifications
    pub async fn list(&self, org_id: Uuid, status: Option<&str>, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetReclassification>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list(org_id, status, asset_id).await
    }

    /// Approve a pending reclassification
    pub async fn approve(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<AssetReclassification> {
        let rc = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reclassification {} not found", id)))?;

        if rc.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve reclassification in '{}' status. Must be 'pending'.", rc.status
            )));
        }
        info!("Approving reclassification {}", rc.reclassification_number);
        self.repository.update_status(id, "approved", Some(approved_by)).await
    }

    /// Complete an approved reclassification (apply changes)
    pub async fn complete(&self, id: Uuid) -> AtlasResult<AssetReclassification> {
        let rc = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reclassification {} not found", id)))?;

        if rc.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete reclassification in '{}' status. Must be 'approved'.", rc.status
            )));
        }
        info!("Completing reclassification {}", rc.reclassification_number);
        self.repository.update_status(id, "completed", None).await
    }

    /// Reject a pending reclassification
    pub async fn reject(&self, id: Uuid) -> AtlasResult<AssetReclassification> {
        let rc = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reclassification {} not found", id)))?;

        if rc.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject reclassification in '{}' status. Must be 'pending'.", rc.status
            )));
        }
        info!("Rejecting reclassification {}", rc.reclassification_number);
        self.repository.update_status(id, "rejected", None).await
    }

    /// Cancel a pending reclassification
    pub async fn cancel(&self, id: Uuid) -> AtlasResult<AssetReclassification> {
        let rc = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reclassification {} not found", id)))?;

        if rc.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel reclassification in '{}' status. Must be 'pending'.", rc.status
            )));
        }
        info!("Cancelling reclassification {}", rc.reclassification_number);
        self.repository.update_status(id, "cancelled", None).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ReclassificationDashboard> {
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
        items: std::sync::Mutex<Vec<AssetReclassification>>,
    }
    impl MockRepo { fn new() -> Self { Self { items: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl AssetReclassificationRepository for MockRepo {
        async fn create(&self, org_id: Uuid, number: &str, asset_id: Uuid, asset_number: Option<&str>, asset_name: Option<&str>, reclassification_type: &str, reason: Option<&str>, from_category_id: Option<Uuid>, from_category_code: Option<&str>, from_asset_type: Option<&str>, from_depreciation_method: Option<&str>, from_useful_life_months: Option<i32>, from_asset_account_code: Option<&str>, from_depr_expense_account_code: Option<&str>, to_category_id: Option<Uuid>, to_category_code: Option<&str>, to_asset_type: Option<&str>, to_depreciation_method: Option<&str>, to_useful_life_months: Option<i32>, to_asset_account_code: Option<&str>, to_depr_expense_account_code: Option<&str>, effective_date: chrono::NaiveDate, amortization_adjustment: Option<&str>, status: &str, notes: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<AssetReclassification> {
            let rc = AssetReclassification {
                id: Uuid::new_v4(), organization_id: org_id, reclassification_number: number.into(),
                asset_id, asset_number: asset_number.map(Into::into), asset_name: asset_name.map(Into::into),
                reclassification_type: reclassification_type.into(), reason: reason.map(Into::into),
                from_category_id, from_category_code: from_category_code.map(Into::into),
                from_asset_type: from_asset_type.map(Into::into),
                from_depreciation_method: from_depreciation_method.map(Into::into),
                from_useful_life_months, from_asset_account_code: from_asset_account_code.map(Into::into),
                from_depr_expense_account_code: from_depr_expense_account_code.map(Into::into),
                to_category_id, to_category_code: to_category_code.map(Into::into),
                to_asset_type: to_asset_type.map(Into::into),
                to_depreciation_method: to_depreciation_method.map(Into::into),
                to_useful_life_months, to_asset_account_code: to_asset_account_code.map(Into::into),
                to_depr_expense_account_code: to_depr_expense_account_code.map(Into::into),
                effective_date, amortization_adjustment: amortization_adjustment.map(Into::into),
                status: status.into(), approved_by: None, notes: notes.map(Into::into),
                metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.items.lock().unwrap().push(rc.clone());
            Ok(rc)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetReclassification>> {
            Ok(self.items.lock().unwrap().iter().find(|r| r.id == id).cloned())
        }
        async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetReclassification>> {
            Ok(self.items.lock().unwrap().iter().find(|r| r.organization_id == org_id && r.reclassification_number == number).cloned())
        }
        async fn list(&self, org_id: Uuid, status: Option<&str>, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetReclassification>> {
            Ok(self.items.lock().unwrap().iter()
                .filter(|r| r.organization_id == org_id
                    && (status.is_none() || r.status == status.unwrap())
                    && (asset_id.is_none() || r.asset_id == asset_id.unwrap()))
                .cloned().collect())
        }
        async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<AssetReclassification> {
            let mut all = self.items.lock().unwrap();
            let rc = all.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            rc.status = status.into();
            if let Some(ab) = approved_by { rc.approved_by = Some(ab); }
            rc.updated_at = Utc::now();
            Ok(rc.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ReclassificationDashboard> {
            let all = self.items.lock().unwrap();
            let org_items: Vec<_> = all.iter().filter(|r| r.organization_id == org_id).collect();
            Ok(ReclassificationDashboard {
                organization_id: org_id,
                total_reclassifications: org_items.len() as i32,
                pending_reclassifications: org_items.iter().filter(|r| r.status == "pending").count() as i32,
                approved_reclassifications: org_items.iter().filter(|r| r.status == "approved").count() as i32,
                completed_reclassifications: org_items.iter().filter(|r| r.status == "completed").count() as i32,
                rejected_reclassifications: org_items.iter().filter(|r| r.status == "rejected").count() as i32,
                by_type: serde_json::json!([]),
            })
        }
    }

    fn eng() -> AssetReclassificationEngine { AssetReclassificationEngine::new(Arc::new(MockRepo::new())) }

    fn today() -> chrono::NaiveDate { chrono::Utc::now().date_naive() }

    #[test]
    fn test_valid_reclass_types() {
        assert!(VALID_RECLASS_TYPES.contains(&"category_change"));
        assert!(VALID_RECLASS_TYPES.contains(&"depreciation_change"));
        assert!(VALID_RECLASS_TYPES.contains(&"account_change"));
        assert!(VALID_RECLASS_TYPES.contains(&"comprehensive"));
    }

    #[test]
    fn test_valid_amortization_adjustments() {
        assert!(VALID_AMORTIZATION_ADJUSTMENTS.contains(&"catchup"));
        assert!(VALID_AMORTIZATION_ADJUSTMENTS.contains(&"prospectively"));
        assert!(VALID_AMORTIZATION_ADJUSTMENTS.contains(&"restatement"));
    }

    #[tokio::test]
    async fn test_create_valid() {
        let rc = eng().create(
            Uuid::new_v4(), Uuid::new_v4(), Some("AST-001"), Some("Laptop"),
            "category_change", Some("Reclassified to IT equipment"),
            Some("GEN_EQUIP"), None, None, None, None,
            Some("IT_EQUIP"), None, None, None, None,
            today(), Some("prospectively"), None, None,
        ).await.unwrap();
        assert_eq!(rc.status, "pending");
        assert_eq!(rc.reclassification_type, "category_change");
    }

    #[tokio::test]
    async fn test_create_invalid_type() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "invalid_type", None, None, None, None, None, None, None, None, None, None, None,
            today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_amortization() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "category_change", None, None, None, None, None, None, None, None, None, None, None,
            today(), Some("invalid"), None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_useful_life() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "depreciation_change", None, None, None, None, None, None, None, None, None, Some(-12), None,
            today(), None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_approve() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let approved = e.approve(rc.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");
    }

    #[tokio::test]
    async fn test_approve_not_pending() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let _ = e.approve(rc.id, Uuid::new_v4()).await.unwrap();
        assert!(e.approve(rc.id, Uuid::new_v4()).await.is_err());
    }

    #[tokio::test]
    async fn test_complete() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let _ = e.approve(rc.id, Uuid::new_v4()).await.unwrap();
        let completed = e.complete(rc.id).await.unwrap();
        assert_eq!(completed.status, "completed");
    }

    #[tokio::test]
    async fn test_complete_not_approved() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        assert!(e.complete(rc.id).await.is_err());
    }

    #[tokio::test]
    async fn test_reject() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let rejected = e.reject(rc.id).await.unwrap();
        assert_eq!(rejected.status, "rejected");
    }

    #[tokio::test]
    async fn test_reject_not_pending() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let _ = e.approve(rc.id, Uuid::new_v4()).await.unwrap();
        assert!(e.reject(rc.id).await.is_err());
    }

    #[tokio::test]
    async fn test_cancel() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let cancelled = e.cancel(rc.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_not_pending() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), None, None, "category_change", None, None, None, None, None, None, None, None, None, None, None, today(), None, None, None).await.unwrap();
        let _ = e.approve(rc.id, Uuid::new_v4()).await.unwrap();
        assert!(e.cancel(rc.id).await.is_err());
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_valid() {
        assert!(eng().list(Uuid::new_v4(), Some("pending"), None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_workflow_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let rc = e.create(org, Uuid::new_v4(), Some("AST-100"), Some("Server"),
            "comprehensive", Some("Full reclass"), None, None, None, None, None,
            None, None, None, None, None, today(), Some("catchup"), Some("notes"), None,
        ).await.unwrap();
        assert_eq!(rc.status, "pending");

        let approved = e.approve(rc.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");

        let completed = e.complete(rc.id).await.unwrap();
        assert_eq!(completed.status, "completed");

        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_reclassifications, 1);
        assert_eq!(dash.completed_reclassifications, 1);
    }
}
