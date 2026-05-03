//! Statistical Accounting Module
//!
//! Oracle Fusion Cloud ERP-inspired Statistical Accounting.
//! Tracks non-monetary statistical data alongside financial data for
//! reporting and allocation purposes (e.g., headcount, square footage,
//! units produced, machine hours).
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Statistical Accounting

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

const VALID_STAT_TYPES: &[&str] = &["headcount", "square_footage", "units_produced", "machine_hours", "labor_hours", "transactions", "vehicles", "lines_of_code", "custom"];
const VALID_UNITS: &[&str] = &["people", "sqft", "sqm", "units", "hours", "transactions", "vehicles", "kloc", "each"];
const VALID_ENTRY_STATUSES: &[&str] = &["draft", "posted", "reversed"];

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalUnit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub stat_type: String,
    pub unit_of_measure: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entry_number: String,
    pub statistical_unit_id: Uuid,
    pub statistical_unit_code: Option<String>,
    pub account_code: Option<String>,
    pub dimension1: Option<String>,
    pub dimension2: Option<String>,
    pub dimension3: Option<String>,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub quantity: String,
    pub unit_cost: Option<String>,
    pub extended_amount: Option<String>,
    pub status: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalBalance {
    pub statistical_unit_id: Uuid,
    pub statistical_unit_code: String,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub beginning_balance: String,
    pub period_activity: String,
    pub ending_balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalDashboard {
    pub organization_id: Uuid,
    pub total_units: i32,
    pub active_units: i32,
    pub total_entries: i32,
    pub posted_entries: i32,
    pub by_type: serde_json::Value,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait StatisticalAccountingRepository: Send + Sync {
    // Units
    async fn create_unit(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, stat_type: &str, unit_of_measure: &str, created_by: Option<Uuid>) -> AtlasResult<StatisticalUnit>;
    async fn get_unit(&self, id: Uuid) -> AtlasResult<Option<StatisticalUnit>>;
    async fn get_unit_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<StatisticalUnit>>;
    async fn list_units(&self, org_id: Uuid, stat_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<StatisticalUnit>>;
    async fn deactivate_unit(&self, id: Uuid) -> AtlasResult<StatisticalUnit>;
    async fn activate_unit(&self, id: Uuid) -> AtlasResult<StatisticalUnit>;

    // Entries
    async fn create_entry(&self,
        org_id: Uuid, entry_number: &str, statistical_unit_id: Uuid, statistical_unit_code: Option<&str>,
        account_code: Option<&str>, dimension1: Option<&str>, dimension2: Option<&str>, dimension3: Option<&str>,
        fiscal_year: i32, period_number: i32, quantity: &str,
        unit_cost: Option<&str>, extended_amount: Option<&str>,
        status: &str, source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<StatisticalEntry>;
    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<StatisticalEntry>>;
    async fn list_entries(&self, org_id: Uuid, unit_id: Option<Uuid>, fiscal_year: Option<i32>, period: Option<i32>, status: Option<&str>) -> AtlasResult<Vec<StatisticalEntry>>;
    async fn update_entry_status(&self, id: Uuid, status: &str) -> AtlasResult<StatisticalEntry>;

    // Balance
    async fn get_balance(&self, org_id: Uuid, unit_id: Uuid, fiscal_year: i32, period: i32) -> AtlasResult<Option<StatisticalBalance>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<StatisticalDashboard>;
}

/// PostgreSQL stub
pub struct PostgresStatisticalAccountingRepository { pool: PgPool }
impl PostgresStatisticalAccountingRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl StatisticalAccountingRepository for PostgresStatisticalAccountingRepository {
    async fn create_unit(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: &str, _: Option<Uuid>) -> AtlasResult<StatisticalUnit> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_unit(&self, _: Uuid) -> AtlasResult<Option<StatisticalUnit>> { Ok(None) }
    async fn get_unit_by_code(&self, _: Uuid, _: &str) -> AtlasResult<Option<StatisticalUnit>> { Ok(None) }
    async fn list_units(&self, _: Uuid, _: Option<&str>, _: Option<bool>) -> AtlasResult<Vec<StatisticalUnit>> { Ok(vec![]) }
    async fn deactivate_unit(&self, _: Uuid) -> AtlasResult<StatisticalUnit> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn activate_unit(&self, _: Uuid) -> AtlasResult<StatisticalUnit> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn create_entry(&self, _: Uuid, _: &str, _: Uuid, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: i32, _: i32, _: &str, _: Option<&str>, _: Option<&str>, _: &str, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<StatisticalEntry> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_entry(&self, _: Uuid) -> AtlasResult<Option<StatisticalEntry>> { Ok(None) }
    async fn list_entries(&self, _: Uuid, _: Option<Uuid>, _: Option<i32>, _: Option<i32>, _: Option<&str>) -> AtlasResult<Vec<StatisticalEntry>> { Ok(vec![]) }
    async fn update_entry_status(&self, _: Uuid, _: &str) -> AtlasResult<StatisticalEntry> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_balance(&self, _: Uuid, _: Uuid, _: i32, _: i32) -> AtlasResult<Option<StatisticalBalance>> { Ok(None) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<StatisticalDashboard> {
        Ok(StatisticalDashboard { organization_id: org_id, total_units: 0, active_units: 0, total_entries: 0, posted_entries: 0, by_type: serde_json::json!([]) })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct StatisticalAccountingEngine {
    repository: Arc<dyn StatisticalAccountingRepository>,
}

impl StatisticalAccountingEngine {
    pub fn new(repository: Arc<dyn StatisticalAccountingRepository>) -> Self {
        Self { repository }
    }

    // ── Unit operations ──

    pub async fn create_unit(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        stat_type: &str, unit_of_measure: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<StatisticalUnit> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if !VALID_STAT_TYPES.contains(&stat_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid stat type '{}'. Must be one of: {}", stat_type, VALID_STAT_TYPES.join(", ")
            )));
        }
        if !VALID_UNITS.contains(&unit_of_measure) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid unit of measure '{}'. Must be one of: {}", unit_of_measure, VALID_UNITS.join(", ")
            )));
        }
        if self.repository.get_unit_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Statistical unit '{}' already exists", code)));
        }
        info!("Creating statistical unit '{}' for org {}", code, org_id);
        self.repository.create_unit(org_id, code, name, description, stat_type, unit_of_measure, created_by).await
    }

    pub async fn get_unit(&self, id: Uuid) -> AtlasResult<Option<StatisticalUnit>> { self.repository.get_unit(id).await }
    pub async fn get_unit_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<StatisticalUnit>> { self.repository.get_unit_by_code(org_id, code).await }

    pub async fn list_units(&self, org_id: Uuid, stat_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<StatisticalUnit>> {
        if let Some(st) = stat_type {
            if !VALID_STAT_TYPES.contains(&st) {
                return Err(AtlasError::ValidationFailed(format!("Invalid stat type '{}'", st)));
            }
        }
        self.repository.list_units(org_id, stat_type, is_active).await
    }

    pub async fn deactivate_unit(&self, id: Uuid) -> AtlasResult<StatisticalUnit> {
        let u = self.repository.get_unit(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Unit {} not found", id)))?;
        if !u.is_active { return Err(AtlasError::ValidationFailed("Already inactive".into())); }
        self.repository.deactivate_unit(id).await
    }

    pub async fn activate_unit(&self, id: Uuid) -> AtlasResult<StatisticalUnit> {
        let u = self.repository.get_unit(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Unit {} not found", id)))?;
        if u.is_active { return Err(AtlasError::ValidationFailed("Already active".into())); }
        self.repository.activate_unit(id).await
    }

    // ── Entry operations ──

    pub async fn create_entry(
        &self, org_id: Uuid, statistical_unit_id: Uuid,
        account_code: Option<&str>, dimension1: Option<&str>, dimension2: Option<&str>, dimension3: Option<&str>,
        fiscal_year: i32, period_number: i32, quantity: &str,
        unit_cost: Option<&str>, source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<StatisticalEntry> {
        let unit = self.repository.get_unit(statistical_unit_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Statistical unit {} not found", statistical_unit_id)))?;
        if !unit.is_active {
            return Err(AtlasError::ValidationFailed("Cannot create entries for inactive statistical unit".into()));
        }
        if fiscal_year <= 0 { return Err(AtlasError::ValidationFailed("Fiscal year must be positive".into())); }
        if period_number <= 0 || period_number > 13 { return Err(AtlasError::ValidationFailed("Period must be 1-13".into())); }
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed("Invalid quantity".into()))?;
        if qty == 0.0 { return Err(AtlasError::ValidationFailed("Quantity cannot be zero".into())); }

        let extended = if let Some(uc) = unit_cost {
            let uc_val: f64 = uc.parse().map_err(|_| AtlasError::ValidationFailed("Invalid unit cost".into()))?;
            Some((qty * uc_val).to_string())
        } else {
            None
        };

        let entry_number = format!("STAT-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating statistical entry {} for unit {}", entry_number, unit.code);
        self.repository.create_entry(
            org_id, &entry_number, statistical_unit_id, Some(&unit.code),
            account_code, dimension1, dimension2, dimension3,
            fiscal_year, period_number, quantity,
            unit_cost, extended.as_deref(),
            "draft", source_type, source_id, source_number,
            description, created_by,
        ).await
    }

    pub async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<StatisticalEntry>> { self.repository.get_entry(id).await }

    pub async fn list_entries(&self, org_id: Uuid, unit_id: Option<Uuid>, fiscal_year: Option<i32>, period: Option<i32>, status: Option<&str>) -> AtlasResult<Vec<StatisticalEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s)));
            }
        }
        self.repository.list_entries(org_id, unit_id, fiscal_year, period, status).await
    }

    pub async fn post_entry(&self, id: Uuid) -> AtlasResult<StatisticalEntry> {
        let entry = self.repository.get_entry(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", id)))?;
        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(format!("Cannot post entry in '{}' status", entry.status)));
        }
        info!("Posting statistical entry {}", entry.entry_number);
        self.repository.update_entry_status(id, "posted").await
    }

    pub async fn reverse_entry(&self, id: Uuid) -> AtlasResult<StatisticalEntry> {
        let entry = self.repository.get_entry(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", id)))?;
        if entry.status != "posted" {
            return Err(AtlasError::WorkflowError(format!("Cannot reverse entry in '{}' status", entry.status)));
        }
        info!("Reversing statistical entry {}", entry.entry_number);
        self.repository.update_entry_status(id, "reversed").await
    }

    pub async fn get_balance(&self, org_id: Uuid, unit_id: Uuid, fiscal_year: i32, period: i32) -> AtlasResult<Option<StatisticalBalance>> {
        self.repository.get_balance(org_id, unit_id, fiscal_year, period).await
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<StatisticalDashboard> {
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
        units: std::sync::Mutex<Vec<StatisticalUnit>>,
        entries: std::sync::Mutex<Vec<StatisticalEntry>>,
    }
    impl MockRepo { fn new() -> Self { Self { units: std::sync::Mutex::new(vec![]), entries: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl StatisticalAccountingRepository for MockRepo {
        async fn create_unit(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, stat_type: &str, uom: &str, created_by: Option<Uuid>) -> AtlasResult<StatisticalUnit> {
            let u = StatisticalUnit {
                id: Uuid::new_v4(), organization_id: org_id, code: code.into(), name: name.into(),
                description: description.map(Into::into), stat_type: stat_type.into(), unit_of_measure: uom.into(),
                is_active: true, metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.units.lock().unwrap().push(u.clone());
            Ok(u)
        }
        async fn get_unit(&self, id: Uuid) -> AtlasResult<Option<StatisticalUnit>> {
            Ok(self.units.lock().unwrap().iter().find(|u| u.id == id).cloned())
        }
        async fn get_unit_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<StatisticalUnit>> {
            Ok(self.units.lock().unwrap().iter().find(|u| u.organization_id == org_id && u.code == code).cloned())
        }
        async fn list_units(&self, org_id: Uuid, stat_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<StatisticalUnit>> {
            Ok(self.units.lock().unwrap().iter()
                .filter(|u| u.organization_id == org_id && (stat_type.is_none() || u.stat_type == stat_type.unwrap()) && (is_active.is_none() || u.is_active == is_active.unwrap()))
                .cloned().collect())
        }
        async fn deactivate_unit(&self, id: Uuid) -> AtlasResult<StatisticalUnit> {
            let mut all = self.units.lock().unwrap();
            let u = all.iter_mut().find(|u| u.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            u.is_active = false; u.updated_at = Utc::now(); Ok(u.clone())
        }
        async fn activate_unit(&self, id: Uuid) -> AtlasResult<StatisticalUnit> {
            let mut all = self.units.lock().unwrap();
            let u = all.iter_mut().find(|u| u.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            u.is_active = true; u.updated_at = Utc::now(); Ok(u.clone())
        }
        async fn create_entry(&self, org_id: Uuid, entry_number: &str, unit_id: Uuid, unit_code: Option<&str>, account_code: Option<&str>, dim1: Option<&str>, dim2: Option<&str>, dim3: Option<&str>, fy: i32, pn: i32, qty: &str, uc: Option<&str>, ext: Option<&str>, status: &str, src_type: Option<&str>, src_id: Option<Uuid>, src_num: Option<&str>, desc: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<StatisticalEntry> {
            let e = StatisticalEntry {
                id: Uuid::new_v4(), organization_id: org_id, entry_number: entry_number.into(),
                statistical_unit_id: unit_id, statistical_unit_code: unit_code.map(Into::into),
                account_code: account_code.map(Into::into), dimension1: dim1.map(Into::into),
                dimension2: dim2.map(Into::into), dimension3: dim3.map(Into::into),
                fiscal_year: fy, period_number: pn, quantity: qty.into(),
                unit_cost: uc.map(Into::into), extended_amount: ext.map(Into::into),
                status: status.into(), source_type: src_type.map(Into::into),
                source_id: src_id, source_number: src_num.map(Into::into), description: desc.map(Into::into),
                metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.entries.lock().unwrap().push(e.clone());
            Ok(e)
        }
        async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<StatisticalEntry>> {
            Ok(self.entries.lock().unwrap().iter().find(|e| e.id == id).cloned())
        }
        async fn list_entries(&self, org_id: Uuid, unit_id: Option<Uuid>, fy: Option<i32>, pn: Option<i32>, status: Option<&str>) -> AtlasResult<Vec<StatisticalEntry>> {
            Ok(self.entries.lock().unwrap().iter()
                .filter(|e| e.organization_id == org_id && (unit_id.is_none() || e.statistical_unit_id == unit_id.unwrap()) && (fy.is_none() || e.fiscal_year == fy.unwrap()) && (pn.is_none() || e.period_number == pn.unwrap()) && (status.is_none() || e.status == status.unwrap()))
                .cloned().collect())
        }
        async fn update_entry_status(&self, id: Uuid, status: &str) -> AtlasResult<StatisticalEntry> {
            let mut all = self.entries.lock().unwrap();
            let e = all.iter_mut().find(|e| e.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            e.status = status.into(); e.updated_at = Utc::now(); Ok(e.clone())
        }
        async fn get_balance(&self, org_id: Uuid, unit_id: Uuid, fy: i32, pn: i32) -> AtlasResult<Option<StatisticalBalance>> {
            let entries = self.entries.lock().unwrap();
            let unit_code = self.units.lock().unwrap().iter().find(|u| u.id == unit_id).map(|u| u.code.clone()).unwrap_or_default();
            let posted: Vec<_> = entries.iter()
                .filter(|e| e.organization_id == org_id && e.statistical_unit_id == unit_id && e.fiscal_year == fy && e.period_number == pn && e.status == "posted")
                .collect();
            if posted.is_empty() { return Ok(None); }
            let activity: f64 = posted.iter().map(|e| e.quantity.parse::<f64>().unwrap_or(0.0)).sum();
            Ok(Some(StatisticalBalance { statistical_unit_id: unit_id, statistical_unit_code: unit_code, fiscal_year: fy, period_number: pn, beginning_balance: "0".into(), period_activity: activity.to_string(), ending_balance: activity.to_string() }))
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<StatisticalDashboard> {
            let units = self.units.lock().unwrap();
            let entries = self.entries.lock().unwrap();
            let org_units: Vec<_> = units.iter().filter(|u| u.organization_id == org_id).collect();
            let org_entries: Vec<_> = entries.iter().filter(|e| e.organization_id == org_id).collect();
            Ok(StatisticalDashboard {
                organization_id: org_id, total_units: org_units.len() as i32,
                active_units: org_units.iter().filter(|u| u.is_active).count() as i32,
                total_entries: org_entries.len() as i32,
                posted_entries: org_entries.iter().filter(|e| e.status == "posted").count() as i32,
                by_type: serde_json::json!([]),
            })
        }
    }

    fn eng() -> StatisticalAccountingEngine { StatisticalAccountingEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_stat_types() { assert!(VALID_STAT_TYPES.contains(&"headcount")); assert!(VALID_STAT_TYPES.contains(&"custom")); }

    #[test]
    fn test_valid_units() { assert!(VALID_UNITS.contains(&"people")); assert!(VALID_UNITS.contains(&"hours")); }

    #[tokio::test]
    async fn test_create_unit_valid() {
        let u = eng().create_unit(Uuid::new_v4(), "HEADCOUNT", "Headcount", Some("Total employees"), "headcount", "people", None).await.unwrap();
        assert_eq!(u.code, "HEADCOUNT");
        assert!(u.is_active);
    }

    #[tokio::test]
    async fn test_create_unit_empty_code() {
        assert!(eng().create_unit(Uuid::new_v4(), "", "Name", None, "headcount", "people", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_unit_invalid_type() {
        assert!(eng().create_unit(Uuid::new_v4(), "CODE", "Name", None, "invalid", "people", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_unit_invalid_uom() {
        assert!(eng().create_unit(Uuid::new_v4(), "CODE", "Name", None, "headcount", "invalid", None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_unit_duplicate() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        assert!(e.create_unit(org, "HC", "Headcount 2", None, "headcount", "people", None).await.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_activate_unit() {
        let e = eng();
        let u = e.create_unit(Uuid::new_v4(), "SQFT", "Square Footage", None, "square_footage", "sqft", None).await.unwrap();
        let deactivated = e.deactivate_unit(u.id).await.unwrap();
        assert!(!deactivated.is_active);
        let activated = e.activate_unit(u.id).await.unwrap();
        assert!(activated.is_active);
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive() {
        let e = eng();
        let u = e.create_unit(Uuid::new_v4(), "SQFT", "Square Footage", None, "square_footage", "sqft", None).await.unwrap();
        let _ = e.deactivate_unit(u.id).await.unwrap();
        assert!(e.deactivate_unit(u.id).await.is_err());
    }

    #[tokio::test]
    async fn test_activate_already_active() {
        let e = eng();
        let u = e.create_unit(Uuid::new_v4(), "SQFT", "Square Footage", None, "square_footage", "sqft", None).await.unwrap();
        assert!(e.activate_unit(u.id).await.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_valid() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let entry = e.create_entry(org, u.id, Some("7000"), Some("IT"), Some("NYC"), None, 2026, 5, "150", Some("65000"), Some("hr_feed"), None, None, Some("May headcount"), None).await.unwrap();
        assert_eq!(entry.status, "draft");
        assert_eq!(entry.quantity, "150");
        assert_eq!(entry.extended_amount.as_deref(), Some("9750000")); // 150 * 65000
    }

    #[tokio::test]
    async fn test_create_entry_inactive_unit() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let _ = e.deactivate_unit(u.id).await.unwrap();
        assert!(e.create_entry(org, u.id, None, None, None, None, 2026, 5, "10", None, None, None, None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_zero_quantity() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        assert!(e.create_entry(org, u.id, None, None, None, None, 2026, 5, "0", None, None, None, None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_invalid_year() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        assert!(e.create_entry(org, u.id, None, None, None, None, 0, 5, "10", None, None, None, None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_entry_invalid_period() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        assert!(e.create_entry(org, u.id, None, None, None, None, 2026, 14, "10", None, None, None, None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_post_entry() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let entry = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "50", None, None, None, None, None, None).await.unwrap();
        let posted = e.post_entry(entry.id).await.unwrap();
        assert_eq!(posted.status, "posted");
    }

    #[tokio::test]
    async fn test_post_entry_not_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let entry = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "50", None, None, None, None, None, None).await.unwrap();
        let _ = e.post_entry(entry.id).await.unwrap();
        assert!(e.post_entry(entry.id).await.is_err());
    }

    #[tokio::test]
    async fn test_reverse_entry() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let entry = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "50", None, None, None, None, None, None).await.unwrap();
        let _ = e.post_entry(entry.id).await.unwrap();
        let reversed = e.reverse_entry(entry.id).await.unwrap();
        assert_eq!(reversed.status, "reversed");
    }

    #[tokio::test]
    async fn test_reverse_entry_not_posted() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let entry = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "50", None, None, None, None, None, None).await.unwrap();
        assert!(e.reverse_entry(entry.id).await.is_err());
    }

    #[tokio::test]
    async fn test_get_balance() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let e1 = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "100", None, None, None, None, None, None).await.unwrap();
        let _ = e.post_entry(e1.id).await.unwrap();
        let e2 = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "50", None, None, None, None, None, None).await.unwrap();
        let _ = e.post_entry(e2.id).await.unwrap();
        let bal = e.get_balance(org, u.id, 2026, 5).await.unwrap().unwrap();
        assert_eq!(bal.period_activity, "150");
    }

    #[tokio::test]
    async fn test_list_entries_invalid_status() {
        assert!(eng().list_entries(Uuid::new_v4(), None, None, None, Some("invalid")).await.is_err());
    }

    #[tokio::test]
    async fn test_list_units_invalid_type() {
        assert!(eng().list_units(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        let entry = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "100", None, None, None, None, None, None).await.unwrap();
        let _ = e.post_entry(entry.id).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_units, 1);
        assert_eq!(dash.active_units, 1);
        assert_eq!(dash.total_entries, 1);
        assert_eq!(dash.posted_entries, 1);
    }

    #[tokio::test]
    async fn test_negative_quantity() {
        let e = eng();
        let org = Uuid::new_v4();
        let u = e.create_unit(org, "HC", "Headcount", None, "headcount", "people", None).await.unwrap();
        // Negative quantities are valid (e.g., headcount reduction)
        let entry = e.create_entry(org, u.id, None, None, None, None, 2026, 5, "-5", None, None, None, None, Some("Layoffs"), None).await.unwrap();
        assert_eq!(entry.quantity, "-5");
    }
}
