//! CIP Capitalization Module
//!
//! Oracle Fusion Cloud ERP-inspired CIP (Construction in Progress) Capitalization.
//! Converts CIP assets to fully capitalized fixed assets after construction
//! or project completion. Handles cost accumulation review and GL posting.
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > CIP Capitalization

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Constants
// ============================================================================

const VALID_STATUSES: &[&str] = &["draft", "submitted", "approved", "capitalized", "reversed", "cancelled"];

const VALID_ASSET_TYPES: &[&str] = &["tangible", "intangible", "leased", "group"];

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipCapitalization {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub capitalization_number: String,
    pub cip_asset_id: Uuid,
    pub cip_asset_number: Option<String>,
    pub cip_asset_name: Option<String>,
    pub capitalized_asset_id: Option<Uuid>,
    pub capitalized_asset_number: Option<String>,
    pub asset_type: String,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    pub book_id: Option<Uuid>,
    pub book_code: Option<String>,
    pub depreciation_method: Option<String>,
    pub useful_life_months: Option<i32>,
    pub salvage_value: String,
    pub total_cip_cost: String,
    pub capitalized_cost: String,
    pub capitalization_date: chrono::NaiveDate,
    pub date_placed_in_service: Option<chrono::NaiveDate>,
    pub location: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub asset_account: Option<String>,
    pub cip_account: Option<String>,
    pub depreciation_account: Option<String>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub posted_to_gl: bool,
    pub gl_batch_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipCostLine {
    pub id: Uuid,
    pub capitalization_id: Uuid,
    pub organization_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub description: Option<String>,
    pub cost_amount: String,
    pub included: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipCapitalizationDashboard {
    pub organization_id: Uuid,
    pub total_capitalizations: i32,
    pub draft_capitalizations: i32,
    pub capitalized_assets: i32,
    pub total_capitalized_cost: String,
    pub pending_review: i32,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait CipCapitalizationRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, capitalization_number: &str,
        cip_asset_id: Uuid, cip_asset_number: Option<&str>, cip_asset_name: Option<&str>,
        asset_type: &str, category_id: Option<Uuid>, category_code: Option<&str>,
        book_id: Option<Uuid>, book_code: Option<&str>,
        depreciation_method: Option<&str>, useful_life_months: Option<i32>,
        salvage_value: &str, total_cip_cost: &str, capitalized_cost: &str,
        capitalization_date: chrono::NaiveDate, date_placed_in_service: Option<chrono::NaiveDate>,
        location: Option<&str>, department_id: Option<Uuid>, department_name: Option<&str>,
        asset_account: Option<&str>, cip_account: Option<&str>, depreciation_account: Option<&str>,
        notes: Option<&str>, status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<CipCapitalization>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<CipCapitalization>>;
    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CipCapitalization>>;
    async fn list(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CipCapitalization>>;
    async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<CipCapitalization>;
    async fn set_capitalized_asset(&self, id: Uuid, asset_id: Uuid, asset_number: &str) -> AtlasResult<CipCapitalization>;
    async fn mark_posted(&self, id: Uuid, gl_batch_id: Uuid) -> AtlasResult<CipCapitalization>;

    async fn add_cost_line(&self, capitalization_id: Uuid, org_id: Uuid,
        source_type: &str, source_id: Option<Uuid>, source_number: Option<&str>,
        description: Option<&str>, cost_amount: &str, included: bool,
    ) -> AtlasResult<CipCostLine>;
    async fn list_cost_lines(&self, capitalization_id: Uuid) -> AtlasResult<Vec<CipCostLine>>;
    async fn toggle_cost_line(&self, id: Uuid, included: bool) -> AtlasResult<CipCostLine>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CipCapitalizationDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresCipCapitalizationRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresCipCapitalizationRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl CipCapitalizationRepository for PostgresCipCapitalizationRepository {
    async fn create(&self, _: Uuid, _: &str, _: Uuid, _: Option<&str>, _: Option<&str>, _: &str, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<i32>, _: &str, _: &str, _: &str, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: &str, _: Option<Uuid>) -> AtlasResult<CipCapitalization> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<CipCapitalization>> { Ok(None) }
    async fn get_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<CipCapitalization>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<CipCapitalization>> { Ok(vec![]) }
    async fn update_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<CipCapitalization> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn set_capitalized_asset(&self, _: Uuid, _: Uuid, _: &str) -> AtlasResult<CipCapitalization> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn mark_posted(&self, _: Uuid, _: Uuid) -> AtlasResult<CipCapitalization> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn add_cost_line(&self, _: Uuid, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: &str, _: bool) -> AtlasResult<CipCostLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_cost_lines(&self, _: Uuid) -> AtlasResult<Vec<CipCostLine>> { Ok(vec![]) }
    async fn toggle_cost_line(&self, _: Uuid, _: bool) -> AtlasResult<CipCostLine> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CipCapitalizationDashboard> {
        Ok(CipCapitalizationDashboard { organization_id: org_id, total_capitalizations: 0, draft_capitalizations: 0, capitalized_assets: 0, total_capitalized_cost: "0".into(), pending_review: 0 })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct CipCapitalizationEngine {
    repository: Arc<dyn CipCapitalizationRepository>,
}

impl CipCapitalizationEngine {
    pub fn new(repository: Arc<dyn CipCapitalizationRepository>) -> Self {
        Self { repository }
    }

    /// Create a new CIP capitalization request
    pub async fn create(
        &self,
        org_id: Uuid,
        cip_asset_id: Uuid,
        cip_asset_number: Option<&str>,
        cip_asset_name: Option<&str>,
        asset_type: &str,
        category_code: Option<&str>,
        book_code: Option<&str>,
        depreciation_method: Option<&str>,
        useful_life_months: Option<i32>,
        salvage_value: &str,
        total_cip_cost: &str,
        capitalization_date: chrono::NaiveDate,
        date_placed_in_service: Option<chrono::NaiveDate>,
        location: Option<&str>,
        department_name: Option<&str>,
        asset_account: Option<&str>,
        cip_account: Option<&str>,
        depreciation_account: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CipCapitalization> {
        if !VALID_ASSET_TYPES.contains(&asset_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid asset type '{}'. Must be one of: {}", asset_type, VALID_ASSET_TYPES.join(", ")
            )));
        }

        if let Some(months) = useful_life_months {
            if months <= 0 {
                return Err(AtlasError::ValidationFailed("Useful life months must be positive".into()));
            }
        }

        let cost_val: f64 = total_cip_cost.parse().map_err(|_| AtlasError::ValidationFailed("Invalid CIP cost".into()))?;
        let salvage_val: f64 = salvage_value.parse().map_err(|_| AtlasError::ValidationFailed("Invalid salvage value".into()))?;

        if cost_val <= 0.0 {
            return Err(AtlasError::ValidationFailed("CIP cost must be positive".into()));
        }
        if salvage_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Salvage value cannot be negative".into()));
        }
        if salvage_val >= cost_val {
            return Err(AtlasError::ValidationFailed("Salvage value cannot exceed cost".into()));
        }

        let capitalized_cost_val = cost_val - salvage_val;
        let cap_number = format!("CIP-CAP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating CIP capitalization {} for CIP asset {}", cap_number, cip_asset_id);

        self.repository.create(
            org_id, &cap_number, cip_asset_id, cip_asset_number, cip_asset_name,
            asset_type, None, category_code, None, book_code,
            depreciation_method, useful_life_months,
            salvage_value, total_cip_cost, &capitalized_cost_val.to_string(),
            capitalization_date, date_placed_in_service, location,
            None, department_name, asset_account, cip_account, depreciation_account,
            notes, "draft", created_by,
        ).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<CipCapitalization>> {
        self.repository.get(id).await
    }

    /// Get by number
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CipCapitalization>> {
        self.repository.get_by_number(org_id, number).await
    }

    /// List capitalizations
    pub async fn list(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CipCapitalization>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s)));
            }
        }
        self.repository.list(org_id, status).await
    }

    /// Add a cost line to the capitalization
    pub async fn add_cost_line(
        &self,
        capitalization_id: Uuid,
        org_id: Uuid,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        cost_amount: &str,
    ) -> AtlasResult<CipCostLine> {
        let cap = self.repository.get(capitalization_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", capitalization_id)))?;
        if cap.status != "draft" {
            return Err(AtlasError::WorkflowError(format!("Cannot add cost lines to '{}' capitalization", cap.status)));
        }
        let cost: f64 = cost_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid cost amount".into()))?;
        if cost <= 0.0 {
            return Err(AtlasError::ValidationFailed("Cost amount must be positive".into()));
        }
        self.repository.add_cost_line(
            capitalization_id, org_id, source_type, source_id, source_number,
            description, cost_amount, true,
        ).await
    }

    /// List cost lines
    pub async fn list_cost_lines(&self, capitalization_id: Uuid) -> AtlasResult<Vec<CipCostLine>> {
        self.repository.list_cost_lines(capitalization_id).await
    }

    /// Toggle cost line inclusion
    pub async fn toggle_cost_line(&self, line_id: Uuid, included: bool) -> AtlasResult<CipCostLine> {
        self.repository.toggle_cost_line(line_id, included).await
    }

    /// Submit for approval
    pub async fn submit(&self, id: Uuid) -> AtlasResult<CipCapitalization> {
        let cap = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", id)))?;
        if cap.status != "draft" {
            return Err(AtlasError::WorkflowError(format!("Cannot submit from '{}' status", cap.status)));
        }
        info!("Submitting CIP capitalization {}", cap.capitalization_number);
        self.repository.update_status(id, "submitted", None).await
    }

    /// Approve capitalization
    pub async fn approve(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<CipCapitalization> {
        let cap = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", id)))?;
        if cap.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!("Cannot approve from '{}' status", cap.status)));
        }
        info!("Approving CIP capitalization {}", cap.capitalization_number);
        self.repository.update_status(id, "approved", Some(approved_by)).await
    }

    /// Capitalize - finalize the process, creating the fixed asset
    pub async fn capitalize(&self, id: Uuid, capitalized_asset_number: &str, gl_batch_id: Option<Uuid>) -> AtlasResult<CipCapitalization> {
        let cap = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", id)))?;
        if cap.status != "approved" {
            return Err(AtlasError::WorkflowError(format!("Cannot capitalize from '{}' status", cap.status)));
        }
        if capitalized_asset_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Capitalized asset number is required".into()));
        }
        info!("Capitalizing CIP asset as {} -> {}", cap.capitalization_number, capitalized_asset_number);
        let asset_id = Uuid::new_v4();
        let _ = self.repository.set_capitalized_asset(id, asset_id, capitalized_asset_number).await?;
        let _ = self.repository.update_status(id, "capitalized", None).await?;
        if let Some(batch_id) = gl_batch_id {
            let _ = self.repository.mark_posted(id, batch_id).await?;
        }
        let cap = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", id)))?;
        Ok(cap)
    }

    /// Reverse a capitalized asset back to CIP
    pub async fn reverse(&self, id: Uuid) -> AtlasResult<CipCapitalization> {
        let cap = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", id)))?;
        if cap.status != "capitalized" {
            return Err(AtlasError::WorkflowError(format!("Cannot reverse from '{}' status", cap.status)));
        }
        info!("Reversing CIP capitalization {}", cap.capitalization_number);
        self.repository.update_status(id, "reversed", None).await
    }

    /// Cancel a draft/submitted capitalization
    pub async fn cancel(&self, id: Uuid) -> AtlasResult<CipCapitalization> {
        let cap = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Capitalization {} not found", id)))?;
        if cap.status != "draft" && cap.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!("Cannot cancel from '{}' status", cap.status)));
        }
        info!("Cancelling CIP capitalization {}", cap.capitalization_number);
        self.repository.update_status(id, "cancelled", None).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CipCapitalizationDashboard> {
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
        items: std::sync::Mutex<Vec<CipCapitalization>>,
        cost_lines: std::sync::Mutex<Vec<CipCostLine>>,
    }
    impl MockRepo { fn new() -> Self { Self { items: std::sync::Mutex::new(vec![]), cost_lines: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl CipCapitalizationRepository for MockRepo {
        async fn create(&self, org_id: Uuid, number: &str, cip_asset_id: Uuid, cip_asset_number: Option<&str>, cip_asset_name: Option<&str>, asset_type: &str, category_id: Option<Uuid>, category_code: Option<&str>, book_id: Option<Uuid>, book_code: Option<&str>, depreciation_method: Option<&str>, useful_life_months: Option<i32>, salvage_value: &str, total_cip_cost: &str, capitalized_cost: &str, capitalization_date: chrono::NaiveDate, date_placed_in_service: Option<chrono::NaiveDate>, location: Option<&str>, department_id: Option<Uuid>, department_name: Option<&str>, asset_account: Option<&str>, cip_account: Option<&str>, depreciation_account: Option<&str>, notes: Option<&str>, status: &str, created_by: Option<Uuid>) -> AtlasResult<CipCapitalization> {
            let c = CipCapitalization {
                id: Uuid::new_v4(), organization_id: org_id, capitalization_number: number.into(),
                cip_asset_id, cip_asset_number: cip_asset_number.map(Into::into), cip_asset_name: cip_asset_name.map(Into::into),
                capitalized_asset_id: None, capitalized_asset_number: None,
                asset_type: asset_type.into(), category_id, category_code: category_code.map(Into::into),
                book_id, book_code: book_code.map(Into::into),
                depreciation_method: depreciation_method.map(Into::into), useful_life_months,
                salvage_value: salvage_value.into(), total_cip_cost: total_cip_cost.into(),
                capitalized_cost: capitalized_cost.into(), capitalization_date, date_placed_in_service,
                location: location.map(Into::into), department_id, department_name: department_name.map(Into::into),
                asset_account: asset_account.map(Into::into), cip_account: cip_account.map(Into::into),
                depreciation_account: depreciation_account.map(Into::into),
                status: status.into(), approved_by: None, posted_to_gl: false, gl_batch_id: None,
                notes: notes.map(Into::into), metadata: serde_json::json!({}),
                created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.items.lock().unwrap().push(c.clone());
            Ok(c)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<CipCapitalization>> {
            Ok(self.items.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }
        async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CipCapitalization>> {
            Ok(self.items.lock().unwrap().iter().find(|c| c.organization_id == org_id && c.capitalization_number == number).cloned())
        }
        async fn list(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CipCapitalization>> {
            Ok(self.items.lock().unwrap().iter()
                .filter(|c| c.organization_id == org_id && (status.is_none() || c.status == status.unwrap()))
                .cloned().collect())
        }
        async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<CipCapitalization> {
            let mut all = self.items.lock().unwrap();
            let c = all.iter_mut().find(|c| c.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            c.status = status.into();
            if let Some(ab) = approved_by { c.approved_by = Some(ab); }
            c.updated_at = Utc::now();
            Ok(c.clone())
        }
        async fn set_capitalized_asset(&self, id: Uuid, asset_id: Uuid, asset_number: &str) -> AtlasResult<CipCapitalization> {
            let mut all = self.items.lock().unwrap();
            let c = all.iter_mut().find(|c| c.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            c.capitalized_asset_id = Some(asset_id);
            c.capitalized_asset_number = Some(asset_number.into());
            c.updated_at = Utc::now();
            Ok(c.clone())
        }
        async fn mark_posted(&self, id: Uuid, gl_batch_id: Uuid) -> AtlasResult<CipCapitalization> {
            let mut all = self.items.lock().unwrap();
            let c = all.iter_mut().find(|c| c.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            c.posted_to_gl = true;
            c.gl_batch_id = Some(gl_batch_id);
            c.updated_at = Utc::now();
            Ok(c.clone())
        }
        async fn add_cost_line(&self, cap_id: Uuid, org_id: Uuid, source_type: &str, source_id: Option<Uuid>, source_number: Option<&str>, description: Option<&str>, cost_amount: &str, included: bool) -> AtlasResult<CipCostLine> {
            let line = CipCostLine {
                id: Uuid::new_v4(), capitalization_id: cap_id, organization_id: org_id,
                source_type: source_type.into(), source_id, source_number: source_number.map(Into::into),
                description: description.map(Into::into), cost_amount: cost_amount.into(), included,
                metadata: serde_json::json!({}), created_at: Utc::now(),
            };
            self.cost_lines.lock().unwrap().push(line.clone());
            Ok(line)
        }
        async fn list_cost_lines(&self, cap_id: Uuid) -> AtlasResult<Vec<CipCostLine>> {
            Ok(self.cost_lines.lock().unwrap().iter().filter(|l| l.capitalization_id == cap_id).cloned().collect())
        }
        async fn toggle_cost_line(&self, id: Uuid, included: bool) -> AtlasResult<CipCostLine> {
            let mut all = self.cost_lines.lock().unwrap();
            let line = all.iter_mut().find(|l| l.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            line.included = included;
            Ok(line.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CipCapitalizationDashboard> {
            let all = self.items.lock().unwrap();
            let org_items: Vec<_> = all.iter().filter(|c| c.organization_id == org_id).collect();
            Ok(CipCapitalizationDashboard {
                organization_id: org_id,
                total_capitalizations: org_items.len() as i32,
                draft_capitalizations: org_items.iter().filter(|c| c.status == "draft").count() as i32,
                capitalized_assets: org_items.iter().filter(|c| c.status == "capitalized").count() as i32,
                total_capitalized_cost: org_items.iter().filter(|c| c.status == "capitalized").map(|c| c.capitalized_cost.parse::<f64>().unwrap_or(0.0)).sum::<f64>().to_string(),
                pending_review: org_items.iter().filter(|c| c.status == "submitted").count() as i32,
            })
        }
    }

    fn eng() -> CipCapitalizationEngine { CipCapitalizationEngine::new(Arc::new(MockRepo::new())) }
    fn today() -> chrono::NaiveDate { chrono::Utc::now().date_naive() }

    #[test]
    fn test_valid_statuses() {
        assert_eq!(VALID_STATUSES.len(), 6);
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"capitalized"));
    }

    #[test]
    fn test_valid_asset_types() {
        assert_eq!(VALID_ASSET_TYPES.len(), 4);
        assert!(VALID_ASSET_TYPES.contains(&"tangible"));
    }

    #[tokio::test]
    async fn test_create_valid() {
        let c = eng().create(
            Uuid::new_v4(), Uuid::new_v4(), Some("CIP-001"), Some("Building A Construction"),
            "tangible", Some("BUILDINGS"), Some("CORP_BOOK"), Some("straight_line"), Some(360),
            "50000", "1000000", today(), Some(today()),
            Some("HQ Campus"), Some("Facilities"), Some("GL_ASSET"), Some("GL_CIP"), Some("GL_DEPR"),
            Some("New HQ building"), None,
        ).await.unwrap();
        assert_eq!(c.status, "draft");
        assert_eq!(c.asset_type, "tangible");
        assert_eq!(c.capitalized_cost, "950000"); // 1000000 - 50000
    }

    #[tokio::test]
    async fn test_create_invalid_asset_type() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "invalid", None, None, None, None, "0", "1000", today(), None,
            None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_zero_cost() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "tangible", None, None, None, None, "0", "0", today(), None,
            None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_cost() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "tangible", None, None, None, None, "0", "-1000", today(), None,
            None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_salvage_exceeds_cost() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "tangible", None, None, None, None, "2000", "1000", today(), None,
            None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_salvage() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "tangible", None, None, None, None, "-100", "1000", today(), None,
            None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_useful_life() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "tangible", None, None, None, Some(-12), "0", "1000", today(), None,
            None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_add_cost_line() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let line = e.add_cost_line(c.id, c.organization_id, "ap_invoice", Some(Uuid::new_v4()), Some("INV-001"), Some("Steel beams"), "2500").await.unwrap();
        assert_eq!(line.source_type, "ap_invoice");
        assert_eq!(line.cost_amount, "2500");
        assert!(line.included);
    }

    #[tokio::test]
    async fn test_add_cost_line_not_draft() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        assert!(e.add_cost_line(c.id, c.organization_id, "ap_invoice", None, None, None, "1000").await.is_err());
    }

    #[tokio::test]
    async fn test_add_cost_line_zero_amount() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        assert!(e.add_cost_line(c.id, c.organization_id, "ap_invoice", None, None, None, "0").await.is_err());
    }

    #[tokio::test]
    async fn test_submit_approve_capitalize() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.create(org, Uuid::new_v4(), Some("CIP-BLDG"), Some("New HQ"), "tangible", Some("BLDG"), Some("CORP"), Some("SL"), Some(360), "50000", "500000", today(), Some(today()), None, None, None, None, None, None, None).await.unwrap();
        assert_eq!(c.status, "draft");

        let submitted = e.submit(c.id).await.unwrap();
        assert_eq!(submitted.status, "submitted");

        let approved = e.approve(c.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");

        let capitalized = e.capitalize(c.id, "FA-BLDG-001", Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(capitalized.status, "capitalized");
        assert!(capitalized.posted_to_gl);
        assert_eq!(capitalized.capitalized_asset_number.unwrap(), "FA-BLDG-001");
    }

    #[tokio::test]
    async fn test_submit_not_draft() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        assert!(e.submit(c.id).await.is_err());
    }

    #[tokio::test]
    async fn test_approve_not_submitted() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        assert!(e.approve(c.id, Uuid::new_v4()).await.is_err());
    }

    #[tokio::test]
    async fn test_capitalize_not_approved() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        assert!(e.capitalize(c.id, "FA-001", None).await.is_err());
    }

    #[tokio::test]
    async fn test_capitalize_empty_asset_number() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        let _ = e.approve(c.id, Uuid::new_v4()).await.unwrap();
        assert!(e.capitalize(c.id, "", None).await.is_err());
    }

    #[tokio::test]
    async fn test_reverse() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        let _ = e.approve(c.id, Uuid::new_v4()).await.unwrap();
        let _ = e.capitalize(c.id, "FA-001", None).await.unwrap();
        let reversed = e.reverse(c.id).await.unwrap();
        assert_eq!(reversed.status, "reversed");
    }

    #[tokio::test]
    async fn test_reverse_not_capitalized() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        assert!(e.reverse(c.id).await.is_err());
    }

    #[tokio::test]
    async fn test_cancel_draft() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let cancelled = e.cancel(c.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_submitted() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        let cancelled = e.cancel(c.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_approved() {
        let e = eng();
        let c = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        let _ = e.approve(c.id, Uuid::new_v4()).await.unwrap();
        assert!(e.cancel(c.id).await.is_err());
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid")).await.is_err());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.create(org, Uuid::new_v4(), None, None, "tangible", None, None, None, None, "0", "5000", today(), None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.submit(c.id).await.unwrap();
        let _ = e.approve(c.id, Uuid::new_v4()).await.unwrap();
        let _ = e.capitalize(c.id, "FA-001", None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_capitalizations, 1);
        assert_eq!(dash.capitalized_assets, 1);
    }
}
