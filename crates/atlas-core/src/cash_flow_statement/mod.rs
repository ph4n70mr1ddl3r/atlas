//! Cash Flow Statement Module
//!
//! Oracle Fusion Cloud ERP-inspired Cash Flow Statements.
//! Generates cash flow statements using direct or indirect methods
//! with line-level detail for operating, investing, and financing activities.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Financial Reports > Cash Flow Statements

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

const VALID_METHODS: &[&str] = &["direct", "indirect"];
const VALID_PERIOD_TYPES: &[&str] = &["monthly", "quarterly", "yearly"];
const VALID_STATUSES: &[&str] = &["draft", "calculated", "reviewed", "published", "archived"];
const VALID_LINE_CATEGORIES: &[&str] = &["operating", "investing", "financing"];
const VALID_LINE_TYPES: &[&str] = &["header", "detail", "subtotal", "total"];

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_number: String,
    pub method: String,
    pub period_type: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub opening_cash_balance: String,
    pub operating_cash_flow: String,
    pub investing_cash_flow: String,
    pub financing_cash_flow: String,
    pub net_change_in_cash: String,
    pub closing_cash_balance: String,
    pub exchange_rate_effect: String,
    pub prepared_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatementLine {
    pub id: Uuid,
    pub statement_id: Uuid,
    pub line_number: i32,
    pub category: String,
    pub description: Option<String>,
    pub line_type: String,
    pub amount: String,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    pub is_non_cash: bool,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowDashboard {
    pub total_statements: i32,
    pub draft_statements: i32,
    pub published_statements: i32,
    pub total_operating: String,
    pub total_investing: String,
    pub total_financing: String,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait CashFlowStatementRepository: Send + Sync {
    async fn create_statement(&self, org_id: Uuid, statement_number: &str, method: &str, period_type: &str, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, created_by: Option<Uuid>) -> AtlasResult<CashFlowStatement>;
    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CashFlowStatement>>;
    async fn get_statement_by_number(&self, org_id: Uuid, statement_number: &str) -> AtlasResult<Option<CashFlowStatement>>;
    async fn list_statements(&self, org_id: Uuid, status: Option<&str>, method: Option<&str>) -> AtlasResult<Vec<CashFlowStatement>>;
    async fn update_statement_status(&self, id: Uuid, status: &str, reviewed_by: Option<Uuid>) -> AtlasResult<CashFlowStatement>;
    async fn update_statement_amounts(&self, id: Uuid, opening: &str, operating: &str, investing: &str, financing: &str, net_change: &str, closing: &str, exchange: &str) -> AtlasResult<()>;

    async fn add_line(&self, statement_id: Uuid, line_number: i32, category: &str, description: Option<&str>, line_type: &str, amount: &str, account_range_from: Option<&str>, account_range_to: Option<&str>, is_non_cash: bool, display_order: i32) -> AtlasResult<CashFlowStatementLine>;
    async fn list_lines(&self, statement_id: Uuid) -> AtlasResult<Vec<CashFlowStatementLine>>;
    async fn remove_line(&self, id: Uuid) -> AtlasResult<()>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashFlowDashboard>;
}

/// PostgreSQL stub
pub struct PostgresCashFlowStatementRepository { pool: PgPool }
impl PostgresCashFlowStatementRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl CashFlowStatementRepository for PostgresCashFlowStatementRepository {
    async fn create_statement(&self, _: Uuid, _: &str, _: &str, _: &str, _: chrono::NaiveDate, _: chrono::NaiveDate, _: Option<Uuid>) -> AtlasResult<CashFlowStatement> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_statement(&self, _: Uuid) -> AtlasResult<Option<CashFlowStatement>> { Ok(None) }
    async fn get_statement_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<CashFlowStatement>> { Ok(None) }
    async fn list_statements(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<CashFlowStatement>> { Ok(vec![]) }
    async fn update_statement_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<CashFlowStatement> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_statement_amounts(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn add_line(&self, _: Uuid, _: i32, _: &str, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: bool, _: i32) -> AtlasResult<CashFlowStatementLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_lines(&self, _: Uuid) -> AtlasResult<Vec<CashFlowStatementLine>> { Ok(vec![]) }
    async fn remove_line(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashFlowDashboard> {
        Ok(CashFlowDashboard { total_statements: 0, draft_statements: 0, published_statements: 0, total_operating: "0".into(), total_investing: "0".into(), total_financing: "0".into() })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct CashFlowStatementEngine {
    repository: Arc<dyn CashFlowStatementRepository>,
}

impl CashFlowStatementEngine {
    pub fn new(repository: Arc<dyn CashFlowStatementRepository>) -> Self { Self { repository } }

    pub async fn create_statement(
        &self, org_id: Uuid, statement_number: &str, method: &str, period_type: &str,
        period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, created_by: Option<Uuid>,
    ) -> AtlasResult<CashFlowStatement> {
        if statement_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Statement number is required".into()));
        }
        if !VALID_METHODS.contains(&method) {
            return Err(AtlasError::ValidationFailed(format!("Invalid method '{}'. Must be one of: {}", method, VALID_METHODS.join(", "))));
        }
        if !VALID_PERIOD_TYPES.contains(&period_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid period type '{}'. Must be one of: {}", period_type, VALID_PERIOD_TYPES.join(", "))));
        }
        if period_end <= period_start {
            return Err(AtlasError::ValidationFailed("Period end must be after period start".into()));
        }
        if self.repository.get_statement_by_number(org_id, statement_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Statement '{}' already exists", statement_number)));
        }
        info!("Creating cash flow statement {} for org {}", statement_number, org_id);
        self.repository.create_statement(org_id, statement_number, method, period_type, period_start, period_end, created_by).await
    }

    pub async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CashFlowStatement>> { self.repository.get_statement(id).await }

    pub async fn list_statements(&self, org_id: Uuid, status: Option<&str>, method: Option<&str>) -> AtlasResult<Vec<CashFlowStatement>> {
        if let Some(s) = status { if !VALID_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s))); } }
        if let Some(m) = method { if !VALID_METHODS.contains(&m) { return Err(AtlasError::ValidationFailed(format!("Invalid method '{}'", m))); } }
        self.repository.list_statements(org_id, status, method).await
    }

    pub async fn calculate(&self, id: Uuid) -> AtlasResult<CashFlowStatement> {
        let stmt = self.repository.get_statement(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Statement {} not found", id)))?;
        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(format!("Cannot calculate statement in '{}' status", stmt.status)));
        }
        let lines = self.repository.list_lines(id).await?;
        let mut operating: f64 = 0.0;
        let mut investing: f64 = 0.0;
        let mut financing: f64 = 0.0;
        for line in &lines {
            if line.line_type != "detail" { continue; }
            let amt: f64 = line.amount.parse().unwrap_or(0.0);
            match line.category.as_str() {
                "operating" => operating += amt,
                "investing" => investing += amt,
                "financing" => financing += amt,
                _ => {}
            }
        }
        let opening: f64 = stmt.opening_cash_balance.parse().unwrap_or(0.0);
        let net_change = operating + investing + financing;
        let closing = opening + net_change;
        self.repository.update_statement_amounts(id, &opening.to_string(), &operating.to_string(), &investing.to_string(), &financing.to_string(), &net_change.to_string(), &closing.to_string(), "0").await?;
        self.repository.update_statement_status(id, "calculated", None).await
    }

    pub async fn review(&self, id: Uuid, reviewer_id: Uuid) -> AtlasResult<CashFlowStatement> {
        let stmt = self.repository.get_statement(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Statement {} not found", id)))?;
        if stmt.status != "calculated" {
            return Err(AtlasError::WorkflowError(format!("Cannot review statement in '{}' status", stmt.status)));
        }
        self.repository.update_statement_status(id, "reviewed", Some(reviewer_id)).await
    }

    pub async fn publish(&self, id: Uuid) -> AtlasResult<CashFlowStatement> {
        let stmt = self.repository.get_statement(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Statement {} not found", id)))?;
        if stmt.status != "reviewed" {
            return Err(AtlasError::WorkflowError(format!("Cannot publish statement in '{}' status", stmt.status)));
        }
        self.repository.update_statement_status(id, "published", None).await
    }

    pub async fn archive(&self, id: Uuid) -> AtlasResult<CashFlowStatement> {
        let stmt = self.repository.get_statement(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Statement {} not found", id)))?;
        if stmt.status != "published" {
            return Err(AtlasError::WorkflowError(format!("Cannot archive statement in '{}' status", stmt.status)));
        }
        self.repository.update_statement_status(id, "archived", None).await
    }

    pub async fn add_line(
        &self, statement_id: Uuid, line_number: i32, category: &str, description: Option<&str>,
        line_type: &str, amount: &str, account_range_from: Option<&str>, account_range_to: Option<&str>,
        is_non_cash: bool, display_order: i32,
    ) -> AtlasResult<CashFlowStatementLine> {
        let stmt = self.repository.get_statement(statement_id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Statement {} not found", statement_id)))?;
        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(format!("Cannot add lines to '{}' statement", stmt.status)));
        }
        if !VALID_LINE_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!("Invalid category '{}'. Must be one of: {}", category, VALID_LINE_CATEGORIES.join(", "))));
        }
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid line type '{}'", line_type)));
        }
        if line_number <= 0 {
            return Err(AtlasError::ValidationFailed("Line number must be positive".into()));
        }
        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid amount".into()))?;
        let _ = amt; // validated
        self.repository.add_line(statement_id, line_number, category, description, line_type, amount, account_range_from, account_range_to, is_non_cash, display_order).await
    }

    pub async fn list_lines(&self, statement_id: Uuid) -> AtlasResult<Vec<CashFlowStatementLine>> {
        self.repository.list_lines(statement_id).await
    }

    pub async fn remove_line(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.remove_line(id).await
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashFlowDashboard> {
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
        statements: std::sync::Mutex<Vec<CashFlowStatement>>,
        lines: std::sync::Mutex<Vec<CashFlowStatementLine>>,
    }
    impl MockRepo { fn new() -> Self { Self { statements: std::sync::Mutex::new(vec![]), lines: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl CashFlowStatementRepository for MockRepo {
        async fn create_statement(&self, org_id: Uuid, num: &str, method: &str, pt: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, cb: Option<Uuid>) -> AtlasResult<CashFlowStatement> {
            let s = CashFlowStatement { id: Uuid::new_v4(), organization_id: org_id, statement_number: num.into(), method: method.into(), period_type: pt.into(), period_start: ps, period_end: pe, opening_cash_balance: "0".into(), operating_cash_flow: "0".into(), investing_cash_flow: "0".into(), financing_cash_flow: "0".into(), net_change_in_cash: "0".into(), closing_cash_balance: "0".into(), exchange_rate_effect: "0".into(), prepared_by: cb, reviewed_by: None, status: "draft".into(), metadata: serde_json::json!({}), created_by: cb, created_at: Utc::now(), updated_at: Utc::now() };
            self.statements.lock().unwrap().push(s.clone());
            Ok(s)
        }
        async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CashFlowStatement>> {
            Ok(self.statements.lock().unwrap().iter().find(|s| s.id == id).cloned())
        }
        async fn get_statement_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<CashFlowStatement>> {
            Ok(self.statements.lock().unwrap().iter().find(|s| s.organization_id == org_id && s.statement_number == num).cloned())
        }
        async fn list_statements(&self, org_id: Uuid, status: Option<&str>, method: Option<&str>) -> AtlasResult<Vec<CashFlowStatement>> {
            Ok(self.statements.lock().unwrap().iter().filter(|s| s.organization_id == org_id && (status.is_none() || s.status == status.unwrap()) && (method.is_none() || s.method == method.unwrap())).cloned().collect())
        }
        async fn update_statement_status(&self, id: Uuid, status: &str, rb: Option<Uuid>) -> AtlasResult<CashFlowStatement> {
            let mut all = self.statements.lock().unwrap();
            let s = all.iter_mut().find(|s| s.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            s.status = status.into(); if let Some(r) = rb { s.reviewed_by = Some(r); } s.updated_at = Utc::now(); Ok(s.clone())
        }
        async fn update_statement_amounts(&self, id: Uuid, open: &str, op: &str, inv: &str, fin: &str, nc: &str, close: &str, ex: &str) -> AtlasResult<()> {
            let mut all = self.statements.lock().unwrap();
            if let Some(s) = all.iter_mut().find(|s| s.id == id) { s.opening_cash_balance = open.into(); s.operating_cash_flow = op.into(); s.investing_cash_flow = inv.into(); s.financing_cash_flow = fin.into(); s.net_change_in_cash = nc.into(); s.closing_cash_balance = close.into(); s.exchange_rate_effect = ex.into(); }
            Ok(())
        }
        async fn add_line(&self, sid: Uuid, ln: i32, cat: &str, desc: Option<&str>, lt: &str, amt: &str, arf: Option<&str>, art: Option<&str>, nc: bool, do_: i32) -> AtlasResult<CashFlowStatementLine> {
            let l = CashFlowStatementLine { id: Uuid::new_v4(), statement_id: sid, line_number: ln, category: cat.into(), description: desc.map(Into::into), line_type: lt.into(), amount: amt.into(), account_range_from: arf.map(Into::into), account_range_to: art.map(Into::into), is_non_cash: nc, display_order: do_, metadata: serde_json::json!({}), created_at: Utc::now() };
            self.lines.lock().unwrap().push(l.clone());
            Ok(l)
        }
        async fn list_lines(&self, sid: Uuid) -> AtlasResult<Vec<CashFlowStatementLine>> {
            Ok(self.lines.lock().unwrap().iter().filter(|l| l.statement_id == sid).cloned().collect())
        }
        async fn remove_line(&self, id: Uuid) -> AtlasResult<()> { self.lines.lock().unwrap().retain(|l| l.id != id); Ok(()) }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashFlowDashboard> {
            let stmts = self.statements.lock().unwrap();
            let org_s: Vec<_> = stmts.iter().filter(|s| s.organization_id == org_id).collect();
            Ok(CashFlowDashboard { total_statements: org_s.len() as i32, draft_statements: org_s.iter().filter(|s| s.status == "draft").count() as i32, published_statements: org_s.iter().filter(|s| s.status == "published").count() as i32, total_operating: "0".into(), total_investing: "0".into(), total_financing: "0".into() })
        }
    }

    fn eng() -> CashFlowStatementEngine { CashFlowStatementEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_METHODS.len(), 2);
        assert_eq!(VALID_PERIOD_TYPES.len(), 3);
        assert_eq!(VALID_STATUSES.len(), 5);
        assert_eq!(VALID_LINE_CATEGORIES.len(), 3);
        assert_eq!(VALID_LINE_TYPES.len(), 4);
    }

    #[tokio::test]
    async fn test_create_statement_valid() {
        let s = eng().create_statement(Uuid::new_v4(), "CFS-001", "indirect", "quarterly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(), None).await.unwrap();
        assert_eq!(s.statement_number, "CFS-001");
        assert_eq!(s.status, "draft");
        assert_eq!(s.method, "indirect");
    }

    #[tokio::test]
    async fn test_create_statement_empty_number() {
        assert!(eng().create_statement(Uuid::new_v4(), "", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_statement_invalid_method() {
        assert!(eng().create_statement(Uuid::new_v4(), "CFS-1", "invalid", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_statement_invalid_period_type() {
        assert!(eng().create_statement(Uuid::new_v4(), "CFS-1", "direct", "weekly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_statement_end_before_start() {
        assert!(eng().create_statement(Uuid::new_v4(), "CFS-1", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_statement_duplicate() {
        let e = eng(); let org = Uuid::new_v4();
        let _ = e.create_statement(org, "CFS-DUP", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        assert!(e.create_statement(org, "CFS-DUP", "indirect", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_calculate_statement() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-C", "indirect", "yearly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(), None).await.unwrap();
        let _ = e.add_line(s.id, 1, "operating", Some("Net Income"), "detail", "100000", None, None, false, 1).await.unwrap();
        let _ = e.add_line(s.id, 2, "investing", Some("Equipment"), "detail", "-25000", None, None, false, 2).await.unwrap();
        let _ = e.add_line(s.id, 3, "financing", Some("Loan Proceeds"), "detail", "50000", None, None, false, 3).await.unwrap();
        let s = e.calculate(s.id).await.unwrap();
        assert_eq!(s.status, "calculated");
        assert_eq!(s.operating_cash_flow, "100000");
        assert_eq!(s.investing_cash_flow, "-25000");
        assert_eq!(s.financing_cash_flow, "50000");
        assert_eq!(s.net_change_in_cash, "125000");
    }

    #[tokio::test]
    async fn test_calculate_not_draft() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-C2", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        let _ = e.add_line(s.id, 1, "operating", None, "detail", "1000", None, None, false, 1).await.unwrap();
        let _ = e.calculate(s.id).await.unwrap();
        assert!(e.calculate(s.id).await.is_err()); // already calculated
    }

    #[tokio::test]
    async fn test_review_publish_archive_workflow() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-W", "indirect", "yearly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(), None).await.unwrap();
        let _ = e.add_line(s.id, 1, "operating", None, "detail", "50000", None, None, false, 1).await.unwrap();
        let _ = e.calculate(s.id).await.unwrap();
        let reviewer = Uuid::new_v4();
        let s = e.review(s.id, reviewer).await.unwrap();
        assert_eq!(s.status, "reviewed");
        let s = e.publish(s.id).await.unwrap();
        assert_eq!(s.status, "published");
        let s = e.archive(s.id).await.unwrap();
        assert_eq!(s.status, "archived");
    }

    #[tokio::test]
    async fn test_review_not_calculated() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-R", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        assert!(e.review(s.id, Uuid::new_v4()).await.is_err());
    }

    #[tokio::test]
    async fn test_publish_not_reviewed() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-P", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        assert!(e.publish(s.id).await.is_err());
    }

    #[tokio::test]
    async fn test_add_line_valid() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-L", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        let l = e.add_line(s.id, 1, "operating", Some("Cash from Sales"), "detail", "50000", Some("4000"), Some("4999"), false, 1).await.unwrap();
        assert_eq!(l.category, "operating");
        assert_eq!(l.amount, "50000");
    }

    #[tokio::test]
    async fn test_add_line_invalid_category() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-L2", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        assert!(e.add_line(s.id, 1, "invalid", None, "detail", "100", None, None, false, 1).await.is_err());
    }

    #[tokio::test]
    async fn test_add_line_invalid_line_type() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-L3", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        assert!(e.add_line(s.id, 1, "operating", None, "bad", "100", None, None, false, 1).await.is_err());
    }

    #[tokio::test]
    async fn test_add_line_not_draft() {
        let e = eng(); let org = Uuid::new_v4();
        let s = e.create_statement(org, "CFS-L4", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        let _ = e.add_line(s.id, 1, "operating", None, "detail", "1000", None, None, false, 1).await.unwrap();
        let _ = e.calculate(s.id).await.unwrap();
        assert!(e.add_line(s.id, 2, "investing", None, "detail", "500", None, None, false, 2).await.is_err());
    }

    #[tokio::test]
    async fn test_list_statements_filter() {
        let e = eng(); let org = Uuid::new_v4();
        let _ = e.create_statement(org, "CFS-F1", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        let _ = e.create_statement(org, "CFS-F2", "indirect", "yearly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(), None).await.unwrap();
        let all = e.list_statements(org, None, None).await.unwrap();
        assert_eq!(all.len(), 2);
        let direct = e.list_statements(org, None, Some("direct")).await.unwrap();
        assert_eq!(direct.len(), 1);
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng(); let org = Uuid::new_v4();
        let _ = e.create_statement(org, "CFS-D", "direct", "monthly",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(), None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_statements, 1);
        assert_eq!(dash.draft_statements, 1);
    }
}
