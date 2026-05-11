//! Asset Retirement Module
//!
//! Oracle Fusion Cloud ERP-inspired Fixed Asset Retirement/Disposal.
//! Handles retirement of fixed assets through sale, scrap, donation, or transfer
//! with GL accounting entries for gain/loss on disposal.
//!
//! Key capabilities:
//! - Create retirement requests with automatic gain/loss calculation
//! - Approve, complete (post to GL), reverse, and cancel retirements
//! - Track retirements by type (sale, scrap, donation, transfer, theft, destruction)
//! - Dashboard with summary statistics
//!
//! Oracle Fusion equivalent: Financials > Fixed Assets > Asset Retirements

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Constants
// ============================================================================

const VALID_RETIREMENT_TYPES: &[&str] = &["sale", "scrap", "donation", "transfer", "theft", "destruction"];
const VALID_STATUSES: &[&str] = &["pending", "approved", "completed", "reversed", "cancelled"];

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRetirement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub retirement_number: String,
    pub asset_id: Uuid,
    pub asset_number: Option<String>,
    pub asset_description: Option<String>,
    pub retirement_type: String,
    pub retirement_date: chrono::NaiveDate,
    pub cost: String,
    pub accumulated_depreciation: String,
    pub net_book_value: String,
    pub proceeds: String,
    pub removal_cost: String,
    pub gain_loss_amount: String,
    pub gain_loss_account: Option<String>,
    pub asset_account: Option<String>,
    pub depreciation_account: Option<String>,
    pub proceeds_account: Option<String>,
    pub removal_cost_account: Option<String>,
    pub buyer_name: Option<String>,
    pub reason: Option<String>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub posted_to_gl: bool,
    pub gl_batch_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetirementDashboard {
    pub organization_id: Uuid,
    pub total_retirements: i32,
    pub pending_retirements: i32,
    pub completed_retirements: i32,
    pub total_proceeds: String,
    pub total_gain: String,
    pub total_loss: String,
    pub by_type: serde_json::Value,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait AssetRetirementRepository: Send + Sync {
    async fn create(&self,
        org_id: Uuid, retirement_number: &str,
        asset_id: Uuid, asset_number: Option<&str>, asset_description: Option<&str>,
        retirement_type: &str, retirement_date: chrono::NaiveDate,
        cost: &str, accumulated_depreciation: &str, net_book_value: &str,
        proceeds: &str, removal_cost: &str, gain_loss_amount: &str,
        gain_loss_account: Option<&str>, asset_account: Option<&str>,
        depreciation_account: Option<&str>, proceeds_account: Option<&str>,
        removal_cost_account: Option<&str>, buyer_name: Option<&str>,
        reason: Option<&str>, status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetRetirement>>;
    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetRetirement>>;
    async fn list(&self, org_id: Uuid, status: Option<&str>, retirement_type: Option<&str>) -> AtlasResult<Vec<AssetRetirement>>;
    async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<AssetRetirement>;
    async fn mark_posted(&self, id: Uuid, gl_batch_id: Uuid) -> AtlasResult<AssetRetirement>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RetirementDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresAssetRetirementRepository {
    pool: PgPool,
}

impl PostgresAssetRetirementRepository {
    #[must_use] 
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_retirement(row: &sqlx::postgres::PgRow) -> AssetRetirement {
    AssetRetirement {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        retirement_number: row.get("retirement_number"),
        asset_id: row.get("asset_id"),
        asset_number: row.get("asset_number"),
        asset_description: row.get("asset_description"),
        retirement_type: row.get("retirement_type"),
        retirement_date: row.get("retirement_date"),
        cost: row.get("cost"),
        accumulated_depreciation: row.get("accumulated_depreciation"),
        net_book_value: row.get("net_book_value"),
        proceeds: row.get("proceeds"),
        removal_cost: row.get("removal_cost"),
        gain_loss_amount: row.get("gain_loss_amount"),
        gain_loss_account: row.get("gain_loss_account"),
        asset_account: row.get("asset_account"),
        depreciation_account: row.get("depreciation_account"),
        proceeds_account: row.get("proceeds_account"),
        removal_cost_account: row.get("removal_cost_account"),
        buyer_name: row.get("buyer_name"),
        reason: row.get("reason"),
        status: row.get("status"),
        approved_by: row.get("approved_by"),
        posted_to_gl: row.get("posted_to_gl"),
        gl_batch_id: row.get("gl_batch_id"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl AssetRetirementRepository for PostgresAssetRetirementRepository {
    async fn create(
        &self,
        org_id: Uuid, retirement_number: &str,
        asset_id: Uuid, asset_number: Option<&str>, asset_description: Option<&str>,
        retirement_type: &str, retirement_date: chrono::NaiveDate,
        cost: &str, accumulated_depreciation: &str, net_book_value: &str,
        proceeds: &str, removal_cost: &str, gain_loss_amount: &str,
        gain_loss_account: Option<&str>, asset_account: Option<&str>,
        depreciation_account: Option<&str>, proceeds_account: Option<&str>,
        removal_cost_account: Option<&str>, buyer_name: Option<&str>,
        reason: Option<&str>, status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement> {
        let row = sqlx::query(
            r"INSERT INTO _atlas.asset_retirements
                (organization_id, retirement_number, asset_id, asset_number, asset_description,
                 retirement_type, retirement_date, cost, accumulated_depreciation, net_book_value,
                 proceeds, removal_cost, gain_loss_amount, gain_loss_account, asset_account,
                 depreciation_account, proceeds_account, removal_cost_account, buyer_name,
                 reason, status, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
            RETURNING *",
        )
        .bind(org_id).bind(retirement_number).bind(asset_id)
        .bind(asset_number).bind(asset_description)
        .bind(retirement_type).bind(retirement_date)
        .bind(cost).bind(accumulated_depreciation).bind(net_book_value)
        .bind(proceeds).bind(removal_cost).bind(gain_loss_amount)
        .bind(gain_loss_account).bind(asset_account).bind(depreciation_account)
        .bind(proceeds_account).bind(removal_cost_account).bind(buyer_name)
        .bind(reason).bind(status).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_retirement(&row))
    }

    async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetRetirement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_retirements WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_retirement(&r)))
    }

    async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetRetirement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_retirements WHERE organization_id = $1 AND retirement_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_retirement(&r)))
    }

    async fn list(&self, org_id: Uuid, status: Option<&str>, retirement_type: Option<&str>) -> AtlasResult<Vec<AssetRetirement>> {
        let rows = sqlx::query(
            r"SELECT * FROM _atlas.asset_retirements
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR retirement_type = $3)
            ORDER BY created_at DESC",
        )
        .bind(org_id).bind(status).bind(retirement_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_retirement).collect())
    }

    async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<AssetRetirement> {
        let row = sqlx::query(
            r"UPDATE _atlas.asset_retirements
            SET status = $2, approved_by = COALESCE($3, approved_by), updated_at = now()
            WHERE id = $1
            RETURNING *",
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_retirement(&row))
    }

    async fn mark_posted(&self, id: Uuid, gl_batch_id: Uuid) -> AtlasResult<AssetRetirement> {
        let row = sqlx::query(
            r"UPDATE _atlas.asset_retirements
            SET posted_to_gl = TRUE, gl_batch_id = $2, updated_at = now()
            WHERE id = $1
            RETURNING *",
        )
        .bind(id).bind(gl_batch_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_retirement(&row))
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RetirementDashboard> {
        let row = sqlx::query(
            r"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COALESCE(SUM(proceeds::NUMERIC) FILTER (WHERE status = 'completed'), 0) as total_proceeds,
                COALESCE(SUM(gain_loss_amount::NUMERIC) FILTER (WHERE status = 'completed' AND gain_loss_amount::NUMERIC > 0), 0) as total_gain,
                COALESCE(SUM(gain_loss_amount::NUMERIC) FILTER (WHERE status = 'completed' AND gain_loss_amount::NUMERIC < 0), 0) as total_loss
            FROM _atlas.asset_retirements WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let pending: i64 = row.try_get("pending").unwrap_or(0);
        let completed: i64 = row.try_get("completed").unwrap_or(0);
        let total_proceeds: String = row.try_get("total_proceeds").unwrap_or_else(|_| "0".to_string());
        let total_gain: String = row.try_get("total_gain").unwrap_or_else(|_| "0".to_string());
        let total_loss: String = row.try_get("total_loss").unwrap_or_else(|_| "0".to_string());

        // By-type breakdown
        let type_rows = sqlx::query(
            r"SELECT retirement_type, COUNT(*) as count,
                COALESCE(SUM(proceeds::NUMERIC), 0) as total_proceeds,
                COALESCE(SUM(gain_loss_amount::NUMERIC), 0) as net_gain_loss
            FROM _atlas.asset_retirements
            WHERE organization_id = $1 AND status = 'completed'
            GROUP BY retirement_type
            ORDER BY retirement_type",
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_type: Vec<serde_json::Value> = type_rows.iter().map(|r| {
            serde_json::json!({
                "retirementType": r.get::<String, _>("retirement_type"),
                "count": r.get::<i64, _>("count"),
                "totalProceeds": row.try_get::<String, _>("total_proceeds").unwrap_or_else(|_| "0".to_string()),
                "netGainLoss": row.try_get::<String, _>("net_gain_loss").unwrap_or_else(|_| "0".to_string()),
            })
        }).collect();

        Ok(RetirementDashboard {
            organization_id: org_id,
            total_retirements: total as i32,
            pending_retirements: pending as i32,
            completed_retirements: completed as i32,
            total_proceeds: total_proceeds.clone(),
            total_gain: total_gain.clone(),
            total_loss: total_loss.clone(),
            by_type: serde_json::Value::Array(by_type),
        })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct AssetRetirementEngine {
    repository: Arc<dyn AssetRetirementRepository>,
}

impl AssetRetirementEngine {
    pub fn new(repository: Arc<dyn AssetRetirementRepository>) -> Self {
        Self { repository }
    }

    /// Create a new asset retirement request
    pub async fn create(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        asset_number: Option<&str>,
        asset_description: Option<&str>,
        retirement_type: &str,
        retirement_date: chrono::NaiveDate,
        cost: &str,
        accumulated_depreciation: &str,
        proceeds: &str,
        removal_cost: &str,
        gain_loss_account: Option<&str>,
        asset_account: Option<&str>,
        depreciation_account: Option<&str>,
        proceeds_account: Option<&str>,
        removal_cost_account: Option<&str>,
        buyer_name: Option<&str>,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement> {
        if !VALID_RETIREMENT_TYPES.contains(&retirement_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid retirement type '{}'. Must be one of: {}", retirement_type, VALID_RETIREMENT_TYPES.join(", ")
            )));
        }

        let cost_val: f64 = cost.parse().map_err(|_| AtlasError::ValidationFailed("Invalid cost value".into()))?;
        let acc_dep: f64 = accumulated_depreciation.parse().map_err(|_| AtlasError::ValidationFailed("Invalid accumulated depreciation".into()))?;
        let proceeds_val: f64 = proceeds.parse().map_err(|_| AtlasError::ValidationFailed("Invalid proceeds value".into()))?;
        let removal_val: f64 = removal_cost.parse().map_err(|_| AtlasError::ValidationFailed("Invalid removal cost".into()))?;

        if cost_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Cost cannot be negative".into()));
        }
        if acc_dep < 0.0 {
            return Err(AtlasError::ValidationFailed("Accumulated depreciation cannot be negative".into()));
        }
        if acc_dep > cost_val {
            return Err(AtlasError::ValidationFailed("Accumulated depreciation cannot exceed cost".into()));
        }
        if proceeds_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Proceeds cannot be negative".into()));
        }
        if removal_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Removal cost cannot be negative".into()));
        }

        let nbv = cost_val - acc_dep;
        let gain_loss = proceeds_val - nbv - removal_val;

        let retirement_number = format!("RET-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating asset retirement {} for asset {}", retirement_number, asset_id);

        self.repository.create(
            org_id, &retirement_number,
            asset_id, asset_number, asset_description,
            retirement_type, retirement_date,
            cost, accumulated_depreciation, &nbv.to_string(),
            proceeds, removal_cost, &gain_loss.to_string(),
            gain_loss_account, asset_account, depreciation_account,
            proceeds_account, removal_cost_account, buyer_name,
            reason, "pending", created_by,
        ).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetRetirement>> {
        self.repository.get(id).await
    }

    /// Get by number
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetRetirement>> {
        self.repository.get_by_number(org_id, number).await
    }

    /// List retirements
    pub async fn list(&self, org_id: Uuid, status: Option<&str>, retirement_type: Option<&str>) -> AtlasResult<Vec<AssetRetirement>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid status '{s}'")));
            }
        }
        if let Some(t) = retirement_type {
            if !VALID_RETIREMENT_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!("Invalid retirement type '{t}'")));
            }
        }
        self.repository.list(org_id, status, retirement_type).await
    }

    /// Approve a pending retirement
    pub async fn approve(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<AssetRetirement> {
        let ret = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Retirement {id} not found")))?;
        if ret.status != "pending" {
            return Err(AtlasError::WorkflowError(format!("Cannot approve retirement in '{}' status", ret.status)));
        }
        info!("Approving asset retirement {}", ret.retirement_number);
        self.repository.update_status(id, "approved", Some(approved_by)).await
    }

    /// Complete an approved retirement (post to GL)
    pub async fn complete(&self, id: Uuid, gl_batch_id: Option<Uuid>) -> AtlasResult<AssetRetirement> {
        let ret = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Retirement {id} not found")))?;
        if ret.status != "approved" {
            return Err(AtlasError::WorkflowError(format!("Cannot complete retirement in '{}' status", ret.status)));
        }
        info!("Completing asset retirement {}", ret.retirement_number);
        let updated = self.repository.update_status(id, "completed", None).await?;
        if let Some(batch_id) = gl_batch_id {
            return self.repository.mark_posted(id, batch_id).await;
        }
        Ok(updated)
    }

    /// Reverse a completed retirement
    pub async fn reverse(&self, id: Uuid) -> AtlasResult<AssetRetirement> {
        let ret = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Retirement {id} not found")))?;
        if ret.status != "completed" {
            return Err(AtlasError::WorkflowError(format!("Cannot reverse retirement in '{}' status", ret.status)));
        }
        info!("Reversing asset retirement {}", ret.retirement_number);
        self.repository.update_status(id, "reversed", None).await
    }

    /// Cancel a pending retirement
    pub async fn cancel(&self, id: Uuid) -> AtlasResult<AssetRetirement> {
        let ret = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Retirement {id} not found")))?;
        if ret.status != "pending" {
            return Err(AtlasError::WorkflowError(format!("Cannot cancel retirement in '{}' status", ret.status)));
        }
        info!("Cancelling asset retirement {}", ret.retirement_number);
        self.repository.update_status(id, "cancelled", None).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RetirementDashboard> {
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
        items: std::sync::Mutex<Vec<AssetRetirement>>,
    }
    impl MockRepo { fn new() -> Self { Self { items: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl AssetRetirementRepository for MockRepo {
        async fn create(&self, org_id: Uuid, number: &str, asset_id: Uuid, asset_number: Option<&str>, asset_description: Option<&str>, retirement_type: &str, retirement_date: chrono::NaiveDate, cost: &str, accumulated_depreciation: &str, net_book_value: &str, proceeds: &str, removal_cost: &str, gain_loss_amount: &str, gain_loss_account: Option<&str>, asset_account: Option<&str>, depreciation_account: Option<&str>, proceeds_account: Option<&str>, removal_cost_account: Option<&str>, buyer_name: Option<&str>, reason: Option<&str>, status: &str, created_by: Option<Uuid>) -> AtlasResult<AssetRetirement> {
            let r = AssetRetirement {
                id: Uuid::new_v4(), organization_id: org_id, retirement_number: number.into(),
                asset_id, asset_number: asset_number.map(Into::into), asset_description: asset_description.map(Into::into),
                retirement_type: retirement_type.into(), retirement_date, cost: cost.into(),
                accumulated_depreciation: accumulated_depreciation.into(), net_book_value: net_book_value.into(),
                proceeds: proceeds.into(), removal_cost: removal_cost.into(), gain_loss_amount: gain_loss_amount.into(),
                gain_loss_account: gain_loss_account.map(Into::into), asset_account: asset_account.map(Into::into),
                depreciation_account: depreciation_account.map(Into::into), proceeds_account: proceeds_account.map(Into::into),
                removal_cost_account: removal_cost_account.map(Into::into), buyer_name: buyer_name.map(Into::into),
                reason: reason.map(Into::into), status: status.into(), approved_by: None, posted_to_gl: false,
                gl_batch_id: None, metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.items.lock().unwrap().push(r.clone());
            Ok(r)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<AssetRetirement>> {
            Ok(self.items.lock().unwrap().iter().find(|r| r.id == id).cloned())
        }
        async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AssetRetirement>> {
            Ok(self.items.lock().unwrap().iter().find(|r| r.organization_id == org_id && r.retirement_number == number).cloned())
        }
        async fn list(&self, org_id: Uuid, status: Option<&str>, retirement_type: Option<&str>) -> AtlasResult<Vec<AssetRetirement>> {
            Ok(self.items.lock().unwrap().iter()
                .filter(|r| r.organization_id == org_id
                    && (status.is_none() || r.status == status.unwrap())
                    && (retirement_type.is_none() || r.retirement_type == retirement_type.unwrap()))
                .cloned().collect())
        }
        async fn update_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<AssetRetirement> {
            let mut all = self.items.lock().unwrap();
            let r = all.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            r.status = status.into();
            if let Some(ab) = approved_by { r.approved_by = Some(ab); }
            r.updated_at = Utc::now();
            Ok(r.clone())
        }
        async fn mark_posted(&self, id: Uuid, gl_batch_id: Uuid) -> AtlasResult<AssetRetirement> {
            let mut all = self.items.lock().unwrap();
            let r = all.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            r.posted_to_gl = true;
            r.gl_batch_id = Some(gl_batch_id);
            r.updated_at = Utc::now();
            Ok(r.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RetirementDashboard> {
            let all = self.items.lock().unwrap();
            let org_items: Vec<_> = all.iter().filter(|r| r.organization_id == org_id).collect();
            let completed: Vec<_> = org_items.iter().filter(|r| r.status == "completed").collect();
            let total_proceeds: f64 = completed.iter().map(|r| r.proceeds.parse::<f64>().unwrap_or(0.0)).sum();
            let gains: f64 = completed.iter().map(|r| r.gain_loss_amount.parse::<f64>().unwrap_or(0.0)).filter(|v| *v > 0.0).sum();
            let losses: f64 = completed.iter().map(|r| r.gain_loss_amount.parse::<f64>().unwrap_or(0.0)).filter(|v| *v < 0.0).sum();
            Ok(RetirementDashboard {
                organization_id: org_id,
                total_retirements: org_items.len() as i32,
                pending_retirements: org_items.iter().filter(|r| r.status == "pending").count() as i32,
                completed_retirements: completed.len() as i32,
                total_proceeds: total_proceeds.to_string(),
                total_gain: gains.to_string(),
                total_loss: losses.to_string(),
                by_type: serde_json::json!([]),
            })
        }
    }

    fn eng() -> AssetRetirementEngine { AssetRetirementEngine::new(Arc::new(MockRepo::new())) }
    fn today() -> chrono::NaiveDate { chrono::Utc::now().date_naive() }

    #[test]
    fn test_valid_retirement_types() {
        assert_eq!(VALID_RETIREMENT_TYPES.len(), 6);
        assert!(VALID_RETIREMENT_TYPES.contains(&"sale"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"scrap"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"donation"));
        assert!(VALID_RETIREMENT_TYPES.contains(&"transfer"));
    }

    #[test]
    fn test_valid_statuses() {
        assert_eq!(VALID_STATUSES.len(), 5);
        assert!(VALID_STATUSES.contains(&"pending"));
        assert!(VALID_STATUSES.contains(&"approved"));
        assert!(VALID_STATUSES.contains(&"completed"));
    }

    #[tokio::test]
    async fn test_create_sale_retirement() {
        let r = eng().create(
            Uuid::new_v4(), Uuid::new_v4(), Some("AST-001"), Some("Company Car"),
            "sale", today(), "30000", "18000", "15000", "500",
            Some("GL_GAIN"), Some("GL_ASSET"), Some("GL_DEPR"), Some("GL_CASH"), Some("GL_REMOVAL"),
            Some("Acme Corp"), Some("Upgrading fleet"), None,
        ).await.unwrap();
        assert_eq!(r.retirement_type, "sale");
        assert_eq!(r.status, "pending");
        assert_eq!(r.net_book_value, "12000"); // 30000 - 18000
        assert_eq!(r.gain_loss_amount, "2500"); // 15000 - 12000 - 500
    }

    #[tokio::test]
    async fn test_create_scrap_retirement_loss() {
        let r = eng().create(
            Uuid::new_v4(), Uuid::new_v4(), Some("AST-002"), Some("Old Printer"),
            "scrap", today(), "5000", "3000", "0", "0",
            None, None, None, None, None, None, Some("Broken beyond repair"), None,
        ).await.unwrap();
        assert_eq!(r.net_book_value, "2000"); // 5000 - 3000
        assert_eq!(r.gain_loss_amount, "-2000"); // 0 - 2000 - 0 (loss)
    }

    #[tokio::test]
    async fn test_create_invalid_retirement_type() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "invalid", today(), "1000", "500", "0", "0",
            None, None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_cost() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "sale", today(), "-1000", "500", "0", "0",
            None, None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_proceeds() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "sale", today(), "1000", "500", "-100", "0",
            None, None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_depreciation_exceeds_cost() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "sale", today(), "1000", "1500", "0", "0",
            None, None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_removal_cost() {
        assert!(eng().create(
            Uuid::new_v4(), Uuid::new_v4(), None, None,
            "sale", today(), "1000", "500", "0", "-50",
            None, None, None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_approve() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        let approved = e.approve(r.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");
    }

    #[tokio::test]
    async fn test_approve_not_pending() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.approve(r.id, Uuid::new_v4()).await.unwrap();
        assert!(e.approve(r.id, Uuid::new_v4()).await.is_err());
    }

    #[tokio::test]
    async fn test_complete() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.approve(r.id, Uuid::new_v4()).await.unwrap();
        let completed = e.complete(r.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(completed.status, "completed");
        assert!(completed.posted_to_gl);
    }

    #[tokio::test]
    async fn test_complete_not_approved() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        assert!(e.complete(r.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_reverse() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.approve(r.id, Uuid::new_v4()).await.unwrap();
        let _ = e.complete(r.id, None).await.unwrap();
        let reversed = e.reverse(r.id).await.unwrap();
        assert_eq!(reversed.status, "reversed");
    }

    #[tokio::test]
    async fn test_reverse_not_completed() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        assert!(e.reverse(r.id).await.is_err());
    }

    #[tokio::test]
    async fn test_cancel() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        let cancelled = e.cancel(r.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_not_pending() {
        let e = eng();
        let r = e.create(Uuid::new_v4(), Uuid::new_v4(), None, None, "sale", today(), "1000", "500", "600", "0", None, None, None, None, None, None, None, None).await.unwrap();
        let _ = e.approve(r.id, Uuid::new_v4()).await.unwrap();
        assert!(e.cancel(r.id).await.is_err());
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_invalid_type() {
        assert!(eng().list(Uuid::new_v4(), None, Some("invalid")).await.is_err());
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let r = e.create(
            org, Uuid::new_v4(), Some("AST-100"), Some("Server Rack"),
            "sale", today(), "50000", "30000", "25000", "1000",
            Some("GL_GAIN"), Some("GL_ASSET"), Some("GL_ACC_DEPR"),
            Some("GL_CASH"), Some("GL_REMOVAL"),
            Some("TechBuyer Inc"), Some("Replacing with new model"), None,
        ).await.unwrap();
        assert_eq!(r.status, "pending");
        assert_eq!(r.net_book_value, "20000"); // 50000 - 30000
        assert_eq!(r.gain_loss_amount, "4000"); // 25000 - 20000 - 1000 (gain)

        let approved = e.approve(r.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(approved.status, "approved");

        let completed = e.complete(r.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(completed.status, "completed");
        assert!(completed.posted_to_gl);

        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_retirements, 1);
        assert_eq!(dash.completed_retirements, 1);
    }
}
