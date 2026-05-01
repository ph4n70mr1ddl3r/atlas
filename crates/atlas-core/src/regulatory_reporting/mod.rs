//! Regulatory Reporting Module
//! Oracle Fusion: Financials > Regulatory Reporting

mod engine;
pub use engine::RegulatoryReportingEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegReportTemplate { pub id: Uuid, pub organization_id: Uuid, pub code: String, pub name: String, pub description: Option<String>, pub authority: String, pub report_category: String, pub filing_frequency: String, pub output_format: String, pub row_definitions: serde_json::Value, pub column_definitions: serde_json::Value, pub validation_rules: serde_json::Value, pub is_active: bool, pub metadata: serde_json::Value, pub created_by: Option<Uuid>, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegReport { pub id: Uuid, pub organization_id: Uuid, pub template_id: Uuid, pub template_code: Option<String>, pub report_number: String, pub name: String, pub status: String, pub period_start: chrono::NaiveDate, pub period_end: chrono::NaiveDate, pub authority: String, pub output_format: String, pub total_debits: String, pub total_credits: String, pub line_count: i32, pub generated_at: Option<chrono::DateTime<chrono::Utc>>, pub reviewed_by: Option<Uuid>, pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>, pub approved_by: Option<Uuid>, pub approved_at: Option<chrono::DateTime<chrono::Utc>>, pub submitted_by: Option<Uuid>, pub submitted_at: Option<chrono::DateTime<chrono::Utc>>, pub filing_reference: Option<String>, pub rejection_reason: Option<String>, pub metadata: serde_json::Value, pub created_by: Option<Uuid>, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegReportLine { pub id: Uuid, pub organization_id: Uuid, pub report_id: Uuid, pub line_number: i32, pub row_code: String, pub row_label: String, pub column_code: String, pub column_label: String, pub amount: String, pub description: Option<String>, pub account_range: Option<String>, pub is_subtotal: bool, pub is_total: bool, pub indent_level: i32, pub metadata: serde_json::Value, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegFilingEntry { pub id: Uuid, pub organization_id: Uuid, pub template_id: Option<Uuid>, pub template_code: Option<String>, pub authority: String, pub report_name: String, pub filing_frequency: String, pub period_start: chrono::NaiveDate, pub period_end: chrono::NaiveDate, pub due_date: chrono::NaiveDate, pub status: String, pub assigned_to: Option<Uuid>, pub report_id: Option<Uuid>, pub filed_at: Option<chrono::DateTime<chrono::Utc>>, pub filed_by: Option<Uuid>, pub filing_reference: Option<String>, pub notes: Option<String>, pub metadata: serde_json::Value, pub created_at: chrono::DateTime<chrono::Utc>, pub updated_at: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegReportingDashboard { pub total_templates: i32, pub active_templates: i32, pub total_reports: i32, pub draft_reports: i32, pub pending_review: i32, pub pending_submission: i32, pub submitted_reports: i32, pub overdue_filings: i32, pub upcoming_filings: i32, pub filings_by_authority: serde_json::Value }

#[async_trait]
pub trait RegulatoryReportingRepository: Send + Sync {
    async fn create_template(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, auth: &str, cat: &str, freq: &str, fmt: &str, rows: serde_json::Value, cols: serde_json::Value, rules: serde_json::Value, cb: Option<Uuid>) -> AtlasResult<RegReportTemplate>;
    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RegReportTemplate>>;
    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<RegReportTemplate>>;
    async fn list_templates(&self, org_id: Uuid, auth: Option<&str>, cat: Option<&str>) -> AtlasResult<Vec<RegReportTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn create_report(&self, org_id: Uuid, tid: Uuid, tc: Option<&str>, rn: &str, name: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, auth: &str, fmt: &str, cb: Option<Uuid>) -> AtlasResult<RegReport>;
    async fn get_report(&self, id: Uuid) -> AtlasResult<Option<RegReport>>;
    async fn get_report_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<RegReport>>;
    async fn list_reports(&self, org_id: Uuid, status: Option<&str>, auth: Option<&str>) -> AtlasResult<Vec<RegReport>>;
    async fn update_report_status(&self, id: Uuid, status: &str, rv: Option<Uuid>, ap: Option<Uuid>, sb: Option<Uuid>, fr: Option<&str>, rr: Option<&str>) -> AtlasResult<RegReport>;
    async fn update_report_line_count(&self, id: Uuid, lc: i32, td: &str, tc: &str) -> AtlasResult<()>;
    async fn create_report_line(&self, org_id: Uuid, rid: Uuid, ln: i32, rc: &str, rl: &str, cc: &str, cl: &str, amt: &str, desc: Option<&str>, ar: Option<&str>, sub: bool, tot: bool, ind: i32) -> AtlasResult<RegReportLine>;
    async fn list_report_lines(&self, rid: Uuid) -> AtlasResult<Vec<RegReportLine>>;
    async fn create_filing(&self, org_id: Uuid, tid: Option<Uuid>, tc: Option<&str>, auth: &str, rn: &str, freq: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, dd: chrono::NaiveDate, at: Option<Uuid>) -> AtlasResult<RegFilingEntry>;
    async fn list_filings(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RegFilingEntry>>;
    async fn update_filing_status(&self, id: Uuid, status: &str, rid: Option<Uuid>, fb: Option<Uuid>, fr: Option<&str>) -> AtlasResult<RegFilingEntry>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RegReportingDashboard>;
}

pub struct PostgresRegulatoryReportingRepository { pool: PgPool }
impl PostgresRegulatoryReportingRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl RegulatoryReportingRepository for PostgresRegulatoryReportingRepository {
    async fn create_template(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: &str, _: &str, _: &str, _: serde_json::Value, _: serde_json::Value, _: serde_json::Value, _: Option<Uuid>) -> AtlasResult<RegReportTemplate> { Err(AtlasError::DatabaseError("NI".into())) }
    async fn get_template(&self, _: Uuid, _: &str) -> AtlasResult<Option<RegReportTemplate>> { Ok(None) }
    async fn get_template_by_id(&self, _: Uuid) -> AtlasResult<Option<RegReportTemplate>> { Ok(None) }
    async fn list_templates(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<RegReportTemplate>> { Ok(vec![]) }
    async fn delete_template(&self, _: Uuid, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_report(&self, _: Uuid, _: Uuid, _: Option<&str>, _: &str, _: &str, _: chrono::NaiveDate, _: chrono::NaiveDate, _: &str, _: &str, _: Option<Uuid>) -> AtlasResult<RegReport> { Err(AtlasError::DatabaseError("NI".into())) }
    async fn get_report(&self, _: Uuid) -> AtlasResult<Option<RegReport>> { Ok(None) }
    async fn get_report_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<RegReport>> { Ok(None) }
    async fn list_reports(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<RegReport>> { Ok(vec![]) }
    async fn update_report_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<Uuid>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>) -> AtlasResult<RegReport> { Err(AtlasError::EntityNotFound("M".into())) }
    async fn update_report_line_count(&self, _: Uuid, _: i32, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_report_line(&self, _: Uuid, _: Uuid, _: i32, _: &str, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: bool, _: bool, _: i32) -> AtlasResult<RegReportLine> { Err(AtlasError::DatabaseError("NI".into())) }
    async fn list_report_lines(&self, _: Uuid) -> AtlasResult<Vec<RegReportLine>> { Ok(vec![]) }
    async fn create_filing(&self, _: Uuid, _: Option<Uuid>, _: Option<&str>, _: &str, _: &str, _: &str, _: chrono::NaiveDate, _: chrono::NaiveDate, _: chrono::NaiveDate, _: Option<Uuid>) -> AtlasResult<RegFilingEntry> { Err(AtlasError::DatabaseError("NI".into())) }
    async fn list_filings(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<RegFilingEntry>> { Ok(vec![]) }
    async fn update_filing_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<RegFilingEntry> { Err(AtlasError::EntityNotFound("M".into())) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<RegReportingDashboard> { Ok(RegReportingDashboard { total_templates: 0, active_templates: 0, total_reports: 0, draft_reports: 0, pending_review: 0, pending_submission: 0, submitted_reports: 0, overdue_filings: 0, upcoming_filings: 0, filings_by_authority: serde_json::json!([]) }) }
}
