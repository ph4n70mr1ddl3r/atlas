//! Mass Additions Module
//!
//! Oracle Fusion Cloud ERP-inspired Fixed Assets Mass Additions.
//! Converts AP invoice lines into fixed assets in batch.
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > Mass Additions

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
pub struct MassAddition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub mass_addition_number: String,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub invoice_line_id: Option<Uuid>,
    pub invoice_line_number: Option<i32>,
    pub description: Option<String>,
    pub asset_key: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    pub book_id: Option<Uuid>,
    pub book_code: Option<String>,
    pub asset_type: Option<String>,
    pub depreciation_method: Option<String>,
    pub useful_life_months: Option<i32>,
    pub cost: String,
    pub salvage_value: String,
    pub salvage_value_percent: Option<String>,
    pub asset_account_code: Option<String>,
    pub depr_expense_account_code: Option<String>,
    pub location: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub po_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub date_placed_in_service: Option<chrono::NaiveDate>,
    pub merge_to_id: Option<Uuid>,
    pub merge_to_number: Option<String>,
    pub reject_reason: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassAdditionDashboard {
    pub organization_id: Uuid,
    pub total_lines: i32,
    pub posted_lines: i32,
    pub on_hold_lines: i32,
    pub merged_lines: i32,
    pub rejected_lines: i32,
    pub pending_review_lines: i32,
    pub total_cost: String,
    pub total_by_category: serde_json::Value,
}

// ============================================================================
// Constants
// ============================================================================

const VALID_STATUSES: &[&str] = &[
    "posted", "on_hold", "pending_review", "merged", "rejected", "converted", "split",
];

const VALID_ASSET_TYPES: &[&str] = &[
    "tangible", "intangible", "leased", "cip", "group",
];

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait MassAdditionRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, mass_addition_number: &str,
        invoice_id: Option<Uuid>, invoice_number: Option<&str>,
        invoice_line_id: Option<Uuid>, invoice_line_number: Option<i32>,
        description: Option<&str>, asset_key: Option<&str>,
        category_id: Option<Uuid>, category_code: Option<&str>,
        book_id: Option<Uuid>, book_code: Option<&str>,
        asset_type: Option<&str>, depreciation_method: Option<&str>,
        useful_life_months: Option<i32>, cost: &str,
        salvage_value: &str, salvage_value_percent: Option<&str>,
        asset_account_code: Option<&str>, depr_expense_account_code: Option<&str>,
        location: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        supplier_id: Option<Uuid>, supplier_number: Option<&str>, supplier_name: Option<&str>,
        po_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>, date_placed_in_service: Option<chrono::NaiveDate>,
        status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<MassAddition>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<MassAddition>>;
    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<MassAddition>>;
    async fn list(&self, org_id: Uuid, status: Option<&str>, category_code: Option<&str>) -> AtlasResult<Vec<MassAddition>>;
    async fn update_status(&self, id: Uuid, status: &str, reject_reason: Option<&str>) -> AtlasResult<MassAddition>;
    async fn update_merge(&self, id: Uuid, merge_to_id: Uuid, merge_to_number: &str) -> AtlasResult<MassAddition>;
    async fn update_category_and_book(&self, id: Uuid, category_id: Option<Uuid>, category_code: Option<&str>, book_id: Option<Uuid>, book_code: Option<&str>) -> AtlasResult<MassAddition>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MassAdditionDashboard>;
}

/// PostgreSQL stub implementation
pub struct PostgresMassAdditionRepository { pool: PgPool }
impl PostgresMassAdditionRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl MassAdditionRepository for PostgresMassAdditionRepository {
    async fn create(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: Option<i32>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<i32>, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<chrono::NaiveDate>, _: Option<chrono::NaiveDate>, _: &str, _: Option<Uuid>) -> AtlasResult<MassAddition> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<MassAddition>> { Ok(None) }
    async fn get_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<MassAddition>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<MassAddition>> { Ok(vec![]) }
    async fn update_status(&self, _: Uuid, _: &str, _: Option<&str>) -> AtlasResult<MassAddition> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_merge(&self, _: Uuid, _: Uuid, _: &str) -> AtlasResult<MassAddition> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_category_and_book(&self, _: Uuid, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<MassAddition> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MassAdditionDashboard> {
        Ok(MassAdditionDashboard { organization_id: org_id, total_lines: 0, posted_lines: 0, on_hold_lines: 0, merged_lines: 0, rejected_lines: 0, pending_review_lines: 0, total_cost: "0".into(), total_by_category: serde_json::json!([]) })
    }
}

// ============================================================================
// Engine
// ============================================================================

use std::sync::Arc;
use tracing::info;

pub struct MassAdditionEngine {
    repository: Arc<dyn MassAdditionRepository>,
}

impl MassAdditionEngine {
    pub fn new(repository: Arc<dyn MassAdditionRepository>) -> Self {
        Self { repository }
    }

    /// Create a new mass addition line from an AP invoice
    pub async fn create_from_invoice(
        &self,
        org_id: Uuid,
        invoice_id: Option<Uuid>,
        invoice_number: Option<&str>,
        invoice_line_id: Option<Uuid>,
        invoice_line_number: Option<i32>,
        description: Option<&str>,
        cost: &str,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        category_code: Option<&str>,
        book_code: Option<&str>,
        asset_type: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MassAddition> {
        // Validate cost
        let cost_val: f64 = cost.parse().map_err(|_| AtlasError::ValidationFailed("Cost must be a valid number".into()))?;
        if cost_val <= 0.0 {
            return Err(AtlasError::ValidationFailed("Cost must be positive".into()));
        }

        // Validate asset type if provided
        if let Some(at) = asset_type {
            if !at.is_empty() && !VALID_ASSET_TYPES.contains(&at) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid asset type '{}'. Must be one of: {}", at, VALID_ASSET_TYPES.join(", ")
                )));
            }
        }

        let ma_number = format!("MA-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating mass addition {} for org {}", ma_number, org_id);

        self.repository.create(
            org_id, &ma_number,
            invoice_id, invoice_number,
            invoice_line_id, invoice_line_number,
            description, None, // asset_key
            None, category_code,
            None, book_code,
            asset_type, None, // depreciation_method
            None, cost,
            "0", None, // salvage_value, salvage_value_percent
            None, None, // asset_account_code, depr_expense_account_code
            None, // location
            None, None, // department
            supplier_id, supplier_number, supplier_name,
            None, // po_number
            None, None, // dates
            "posted", created_by,
        ).await
    }

    /// Get a mass addition by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<MassAddition>> {
        self.repository.get(id).await
    }

    /// Get a mass addition by number
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<MassAddition>> {
        self.repository.get_by_number(org_id, number).await
    }

    /// List mass additions with optional filters
    pub async fn list(&self, org_id: Uuid, status: Option<&str>, category_code: Option<&str>) -> AtlasResult<Vec<MassAddition>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list(org_id, status, category_code).await
    }

    /// Hold a mass addition (prevent conversion)
    pub async fn hold(&self, id: Uuid) -> AtlasResult<MassAddition> {
        let ma = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mass addition {} not found", id)))?;

        if ma.status != "posted" && ma.status != "pending_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot hold mass addition in '{}' status", ma.status
            )));
        }
        info!("Holding mass addition {}", ma.mass_addition_number);
        self.repository.update_status(id, "on_hold", None).await
    }

    /// Release a mass addition from hold
    pub async fn release(&self, id: Uuid) -> AtlasResult<MassAddition> {
        let ma = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mass addition {} not found", id)))?;

        if ma.status != "on_hold" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot release mass addition in '{}' status. Must be 'on_hold'.", ma.status
            )));
        }
        info!("Releasing mass addition {}", ma.mass_addition_number);
        self.repository.update_status(id, "posted", None).await
    }

    /// Reject a mass addition
    pub async fn reject(&self, id: Uuid, reason: &str) -> AtlasResult<MassAddition> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Reject reason is required".into()));
        }
        let ma = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mass addition {} not found", id)))?;

        if ma.status == "converted" || ma.status == "rejected" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject mass addition in '{}' status", ma.status
            )));
        }
        info!("Rejecting mass addition {}: {}", ma.mass_addition_number, reason);
        self.repository.update_status(id, "rejected", Some(reason)).await
    }

    /// Merge mass addition into another
    pub async fn merge(&self, id: Uuid, merge_to_id: Uuid) -> AtlasResult<MassAddition> {
        let ma = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mass addition {} not found", id)))?;

        let target = self.repository.get(merge_to_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Target mass addition {} not found", merge_to_id)))?;

        if ma.organization_id != target.organization_id {
            return Err(AtlasError::ValidationFailed("Cannot merge across organizations".into()));
        }
        if ma.status == "converted" || ma.status == "rejected" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot merge mass addition in '{}' status", ma.status
            )));
        }
        if target.status == "converted" || target.status == "rejected" {
            return Err(AtlasError::ValidationFailed("Cannot merge into a converted/rejected target".into()));
        }

        info!("Merging mass addition {} into {}", ma.mass_addition_number, target.mass_addition_number);
        self.repository.update_merge(id, target.id, &target.mass_addition_number).await
    }

    /// Update category and book for a mass addition
    pub async fn update_category_and_book(
        &self,
        id: Uuid,
        category_id: Option<Uuid>,
        category_code: Option<&str>,
        book_id: Option<Uuid>,
        book_code: Option<&str>,
    ) -> AtlasResult<MassAddition> {
        let ma = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mass addition {} not found", id)))?;

        if ma.status == "converted" {
            return Err(AtlasError::WorkflowError("Cannot modify a converted mass addition".into()));
        }

        info!("Updating category/book for mass addition {}", ma.mass_addition_number);
        self.repository.update_category_and_book(id, category_id, category_code, book_id, book_code).await
    }

    /// Convert mass addition to a fixed asset (mark as converted)
    pub async fn convert(&self, id: Uuid) -> AtlasResult<MassAddition> {
        let ma = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mass addition {} not found", id)))?;

        if ma.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot convert mass addition in '{}' status. Must be 'posted'.", ma.status
            )));
        }

        // Validate required fields for conversion
        if ma.category_code.is_none() || ma.category_code.as_ref().map(|c| c.is_empty()).unwrap_or(true) {
            return Err(AtlasError::ValidationFailed("Category code is required for conversion".into()));
        }
        if ma.book_code.is_none() || ma.book_code.as_ref().map(|c| c.is_empty()).unwrap_or(true) {
            return Err(AtlasError::ValidationFailed("Book code is required for conversion".into()));
        }

        info!("Converting mass addition {} to fixed asset", ma.mass_addition_number);
        self.repository.update_status(id, "converted", None).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MassAdditionDashboard> {
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
        additions: std::sync::Mutex<Vec<MassAddition>>,
    }
    impl MockRepo { fn new() -> Self { Self { additions: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl MassAdditionRepository for MockRepo {
        async fn create(&self, org_id: Uuid, ma_number: &str, invoice_id: Option<Uuid>, invoice_number: Option<&str>, invoice_line_id: Option<Uuid>, invoice_line_number: Option<i32>, description: Option<&str>, asset_key: Option<&str>, category_id: Option<Uuid>, category_code: Option<&str>, book_id: Option<Uuid>, book_code: Option<&str>, asset_type: Option<&str>, depreciation_method: Option<&str>, useful_life_months: Option<i32>, cost: &str, salvage_value: &str, salvage_value_percent: Option<&str>, asset_account_code: Option<&str>, depr_expense_account_code: Option<&str>, location: Option<&str>, department_id: Option<Uuid>, department_name: Option<&str>, supplier_id: Option<Uuid>, supplier_number: Option<&str>, supplier_name: Option<&str>, po_number: Option<&str>, invoice_date: Option<chrono::NaiveDate>, date_placed_in_service: Option<chrono::NaiveDate>, status: &str, created_by: Option<Uuid>) -> AtlasResult<MassAddition> {
            let ma = MassAddition {
                id: Uuid::new_v4(), organization_id: org_id, mass_addition_number: ma_number.into(),
                invoice_id, invoice_number: invoice_number.map(Into::into), invoice_line_id, invoice_line_number,
                description: description.map(Into::into), asset_key: asset_key.map(Into::into),
                category_id, category_code: category_code.map(Into::into),
                book_id, book_code: book_code.map(Into::into),
                asset_type: asset_type.map(Into::into), depreciation_method: depreciation_method.map(Into::into),
                useful_life_months, cost: cost.into(), salvage_value: salvage_value.into(),
                salvage_value_percent: salvage_value_percent.map(Into::into),
                asset_account_code: asset_account_code.map(Into::into),
                depr_expense_account_code: depr_expense_account_code.map(Into::into),
                location: location.map(Into::into), department_id, department_name: department_name.map(Into::into),
                supplier_id, supplier_number: supplier_number.map(Into::into),
                supplier_name: supplier_name.map(Into::into), po_number: po_number.map(Into::into),
                invoice_date, date_placed_in_service,
                merge_to_id: None, merge_to_number: None, reject_reason: None,
                status: status.into(), metadata: serde_json::json!({}),
                created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.additions.lock().unwrap().push(ma.clone());
            Ok(ma)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<MassAddition>> {
            Ok(self.additions.lock().unwrap().iter().find(|m| m.id == id).cloned())
        }
        async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<MassAddition>> {
            Ok(self.additions.lock().unwrap().iter().find(|m| m.organization_id == org_id && m.mass_addition_number == number).cloned())
        }
        async fn list(&self, org_id: Uuid, status: Option<&str>, _category: Option<&str>) -> AtlasResult<Vec<MassAddition>> {
            Ok(self.additions.lock().unwrap().iter()
                .filter(|m| m.organization_id == org_id && (status.is_none() || m.status == status.unwrap()))
                .cloned().collect())
        }
        async fn update_status(&self, id: Uuid, status: &str, reject_reason: Option<&str>) -> AtlasResult<MassAddition> {
            let mut all = self.additions.lock().unwrap();
            let ma = all.iter_mut().find(|m| m.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ma.status = status.into();
            ma.reject_reason = reject_reason.map(Into::into);
            ma.updated_at = Utc::now();
            Ok(ma.clone())
        }
        async fn update_merge(&self, id: Uuid, merge_to_id: Uuid, merge_to_number: &str) -> AtlasResult<MassAddition> {
            let mut all = self.additions.lock().unwrap();
            let ma = all.iter_mut().find(|m| m.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ma.merge_to_id = Some(merge_to_id);
            ma.merge_to_number = Some(merge_to_number.into());
            ma.status = "merged".into();
            ma.updated_at = Utc::now();
            Ok(ma.clone())
        }
        async fn update_category_and_book(&self, id: Uuid, category_id: Option<Uuid>, category_code: Option<&str>, book_id: Option<Uuid>, book_code: Option<&str>) -> AtlasResult<MassAddition> {
            let mut all = self.additions.lock().unwrap();
            let ma = all.iter_mut().find(|m| m.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ma.category_id = category_id;
            ma.category_code = category_code.map(Into::into);
            ma.book_id = book_id;
            ma.book_code = book_code.map(Into::into);
            ma.updated_at = Utc::now();
            Ok(ma.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MassAdditionDashboard> {
            let all = self.additions.lock().unwrap();
            let org_mas: Vec<_> = all.iter().filter(|m| m.organization_id == org_id).collect();
            Ok(MassAdditionDashboard {
                organization_id: org_id,
                total_lines: org_mas.len() as i32,
                posted_lines: org_mas.iter().filter(|m| m.status == "posted").count() as i32,
                on_hold_lines: org_mas.iter().filter(|m| m.status == "on_hold").count() as i32,
                merged_lines: org_mas.iter().filter(|m| m.status == "merged").count() as i32,
                rejected_lines: org_mas.iter().filter(|m| m.status == "rejected").count() as i32,
                pending_review_lines: org_mas.iter().filter(|m| m.status == "pending_review").count() as i32,
                total_cost: org_mas.iter().map(|m| m.cost.parse::<f64>().unwrap_or(0.0)).sum::<f64>().to_string(),
                total_by_category: serde_json::json!([]),
            })
        }
    }

    fn eng() -> MassAdditionEngine { MassAdditionEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"posted"));
        assert!(VALID_STATUSES.contains(&"on_hold"));
        assert!(VALID_STATUSES.contains(&"converted"));
        assert!(VALID_STATUSES.contains(&"merged"));
        assert!(VALID_STATUSES.contains(&"rejected"));
    }

    #[test]
    fn test_valid_asset_types() {
        assert!(VALID_ASSET_TYPES.contains(&"tangible"));
        assert!(VALID_ASSET_TYPES.contains(&"intangible"));
        assert!(VALID_ASSET_TYPES.contains(&"leased"));
        assert!(VALID_ASSET_TYPES.contains(&"cip"));
        assert!(VALID_ASSET_TYPES.contains(&"group"));
    }

    #[tokio::test]
    async fn test_create_from_invoice_valid() {
        let ma = eng().create_from_invoice(
            Uuid::new_v4(), Some(Uuid::new_v4()), Some("INV-001"), None, None,
            Some("Laptop computer"), "1500.00", Some(Uuid::new_v4()), Some("SUP-001"),
            Some("TechCorp"), Some("IT_EQUIP"), Some("CORP_BOOK"), Some("tangible"), None,
        ).await.unwrap();
        assert_eq!(ma.status, "posted");
        assert_eq!(ma.cost, "1500.00");
    }

    #[tokio::test]
    async fn test_create_zero_cost() {
        assert!(eng().create_from_invoice(
            Uuid::new_v4(), None, None, None, None, None, "0", None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_cost() {
        assert!(eng().create_from_invoice(
            Uuid::new_v4(), None, None, None, None, None, "-100", None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_cost() {
        assert!(eng().create_from_invoice(
            Uuid::new_v4(), None, None, None, None, None, "abc", None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_asset_type() {
        assert!(eng().create_from_invoice(
            Uuid::new_v4(), None, None, None, None, None, "100", None, None, None, None, None, Some("invalid"), None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_hold_and_release() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        let held = e.hold(ma.id).await.unwrap();
        assert_eq!(held.status, "on_hold");
        let released = e.release(ma.id).await.unwrap();
        assert_eq!(released.status, "posted");
    }

    #[tokio::test]
    async fn test_hold_wrong_status() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        let _ = e.hold(ma.id).await.unwrap();
        assert!(e.hold(ma.id).await.is_err()); // already on_hold
    }

    #[tokio::test]
    async fn test_release_wrong_status() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        assert!(e.release(ma.id).await.is_err()); // posted, not on_hold
    }

    #[tokio::test]
    async fn test_reject() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        let rejected = e.reject(ma.id, "Duplicate").await.unwrap();
        assert_eq!(rejected.status, "rejected");
        assert_eq!(rejected.reject_reason.unwrap(), "Duplicate");
    }

    #[tokio::test]
    async fn test_reject_empty_reason() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        assert!(e.reject(ma.id, "").await.is_err());
    }

    #[tokio::test]
    async fn test_merge() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma1 = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        let ma2 = e.create_from_invoice(org, None, None, None, None, None, "300", None, None, None, None, None, None, None).await.unwrap();
        let merged = e.merge(ma1.id, ma2.id).await.unwrap();
        assert_eq!(merged.status, "merged");
        assert_eq!(merged.merge_to_id.unwrap(), ma2.id);
    }

    #[tokio::test]
    async fn test_merge_not_found() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "500", None, None, None, None, None, None, None).await.unwrap();
        assert!(e.merge(ma.id, Uuid::new_v4()).await.is_err());
    }

    #[tokio::test]
    async fn test_convert() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "5000", None, None, None, Some("IT_EQUIP"), Some("CORP_BOOK"), None, None).await.unwrap();
        let converted = e.convert(ma.id).await.unwrap();
        assert_eq!(converted.status, "converted");
    }

    #[tokio::test]
    async fn test_convert_missing_category() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "5000", None, None, None, None, Some("CORP_BOOK"), None, None).await.unwrap();
        assert!(e.convert(ma.id).await.is_err());
    }

    #[tokio::test]
    async fn test_convert_missing_book() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "5000", None, None, None, Some("IT_EQUIP"), None, None, None).await.unwrap();
        assert!(e.convert(ma.id).await.is_err());
    }

    #[tokio::test]
    async fn test_convert_wrong_status() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "5000", None, None, None, Some("IT_EQUIP"), Some("CORP_BOOK"), None, None).await.unwrap();
        let _ = e.hold(ma.id).await.unwrap();
        assert!(e.convert(ma.id).await.is_err()); // on_hold, not posted
    }

    #[tokio::test]
    async fn test_update_category_and_book() {
        let e = eng();
        let org = Uuid::new_v4();
        let ma = e.create_from_invoice(org, None, None, None, None, None, "5000", None, None, None, None, None, None, None).await.unwrap();
        let updated = e.update_category_and_book(ma.id, Some(Uuid::new_v4()), Some("VEHICLES"), Some(Uuid::new_v4()), Some("TAX_BOOK")).await.unwrap();
        assert_eq!(updated.category_code.unwrap(), "VEHICLES");
        assert_eq!(updated.book_code.unwrap(), "TAX_BOOK");
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_valid() {
        let r = eng().list(Uuid::new_v4(), Some("posted"), None).await;
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create_from_invoice(org, None, None, None, None, None, "5000", None, None, None, None, None, None, None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_lines, 1);
        assert_eq!(dash.posted_lines, 1);
    }
}
