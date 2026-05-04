//! Financial Ratio Analysis Module
//!
//! Oracle Fusion Cloud ERP-inspired Financial Ratio Analysis.
//! Computes and tracks key financial ratios for liquidity, profitability,
//! leverage, efficiency, and market analysis.
//!
//! Oracle Fusion equivalent: Financials > Financial Reporting Center > Ratio Analysis

mod engine;
pub use engine::FinancialRatioEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Ratio categories from Oracle Fusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ratio_code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub formula: String,
    pub numerator_accounts: serde_json::Value,
    pub denominator_accounts: serde_json::Value,
    pub unit: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub snapshot_date: chrono::NaiveDate,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub currency_code: String,
    pub status: String,
    pub total_ratios: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub snapshot_id: Uuid,
    pub ratio_id: Uuid,
    pub ratio_code: String,
    pub ratio_name: String,
    pub category: String,
    pub numerator_value: String,
    pub denominator_value: String,
    pub result_value: String,
    pub unit: String,
    pub previous_value: Option<String>,
    pub change_amount: Option<String>,
    pub change_percent: Option<String>,
    pub trend_direction: Option<String>,
    pub benchmark_value: Option<String>,
    pub status_flag: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioBenchmark {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ratio_id: Uuid,
    pub name: String,
    pub benchmark_value: String,
    pub min_acceptable: Option<String>,
    pub max_acceptable: Option<String>,
    pub industry: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioCategorySummary {
    pub category: String,
    pub ratio_count: i32,
    pub avg_change_percent: String,
    pub improving_count: i32,
    pub declining_count: i32,
    pub stable_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioDashboard {
    pub total_definitions: i32,
    pub total_snapshots: i32,
    pub liquidity_score: Option<String>,
    pub profitability_score: Option<String>,
    pub leverage_score: Option<String>,
    pub efficiency_score: Option<String>,
    pub category_summaries: Vec<RatioCategorySummary>,
}

/// Repository trait
#[async_trait]
pub trait FinancialRatioRepository: Send + Sync {
    async fn create_definition(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, category: &str, formula: &str, numerator_accounts: serde_json::Value, denominator_accounts: serde_json::Value, unit: &str, created_by: Option<Uuid>) -> AtlasResult<RatioDefinition>;
    async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RatioDefinition>>;
    async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<RatioDefinition>>;
    async fn list_definitions(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<RatioDefinition>>;
    async fn delete_definition(&self, id: Uuid) -> AtlasResult<()>;
    async fn create_snapshot(&self, org_id: Uuid, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, currency_code: &str, created_by: Option<Uuid>) -> AtlasResult<RatioSnapshot>;
    async fn get_snapshot(&self, id: Uuid) -> AtlasResult<Option<RatioSnapshot>>;
    async fn list_snapshots(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RatioSnapshot>>;
    async fn update_snapshot_status(&self, id: Uuid, status: &str, total_ratios: i32) -> AtlasResult<RatioSnapshot>;
    async fn create_ratio_result(&self, org_id: Uuid, snapshot_id: Uuid, ratio_id: Uuid, ratio_code: &str, ratio_name: &str, category: &str, numerator_value: &str, denominator_value: &str, result_value: &str, unit: &str, previous_value: Option<&str>, change_amount: Option<&str>, change_percent: Option<&str>, trend_direction: Option<&str>, benchmark_value: Option<&str>, status_flag: Option<&str>) -> AtlasResult<RatioResult>;
    async fn list_ratio_results(&self, snapshot_id: Uuid) -> AtlasResult<Vec<RatioResult>>;
    async fn list_ratio_results_by_category(&self, snapshot_id: Uuid, category: &str) -> AtlasResult<Vec<RatioResult>>;
    async fn create_benchmark(&self, org_id: Uuid, ratio_id: Uuid, name: &str, value: &str, min: Option<&str>, max: Option<&str>, industry: Option<&str>, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>) -> AtlasResult<RatioBenchmark>;
    async fn list_benchmarks(&self, org_id: Uuid, ratio_id: Option<Uuid>) -> AtlasResult<Vec<RatioBenchmark>>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RatioDashboard>;
}

/// PostgreSQL implementation
#[allow(dead_code)]
pub struct PostgresFinancialRatioRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresFinancialRatioRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl FinancialRatioRepository for PostgresFinancialRatioRepository {
    async fn create_definition(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: &str, _: serde_json::Value, _: serde_json::Value, _: &str, _: Option<Uuid>) -> AtlasResult<RatioDefinition> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_definition(&self, _: Uuid, _: &str) -> AtlasResult<Option<RatioDefinition>> { Ok(None) }
    async fn get_definition_by_id(&self, _: Uuid) -> AtlasResult<Option<RatioDefinition>> { Ok(None) }
    async fn list_definitions(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<RatioDefinition>> { Ok(vec![]) }
    async fn delete_definition(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_snapshot(&self, _: Uuid, _: chrono::NaiveDate, _: chrono::NaiveDate, _: &str, _: Option<Uuid>) -> AtlasResult<RatioSnapshot> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_snapshot(&self, _: Uuid) -> AtlasResult<Option<RatioSnapshot>> { Ok(None) }
    async fn list_snapshots(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<RatioSnapshot>> { Ok(vec![]) }
    async fn update_snapshot_status(&self, _: Uuid, _: &str, _: i32) -> AtlasResult<RatioSnapshot> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn create_ratio_result(&self, _: Uuid, _: Uuid, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> AtlasResult<RatioResult> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_ratio_results(&self, _: Uuid) -> AtlasResult<Vec<RatioResult>> { Ok(vec![]) }
    async fn list_ratio_results_by_category(&self, _: Uuid, _: &str) -> AtlasResult<Vec<RatioResult>> { Ok(vec![]) }
    async fn create_benchmark(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>) -> AtlasResult<RatioBenchmark> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_benchmarks(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<Vec<RatioBenchmark>> { Ok(vec![]) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<RatioDashboard> {
        Ok(RatioDashboard {
            total_definitions: 0, total_snapshots: 0,
            liquidity_score: None, profitability_score: None,
            leverage_score: None, efficiency_score: None,
            category_summaries: vec![],
        })
    }
}
