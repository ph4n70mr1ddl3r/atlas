//! Regulatory Reporting Engine
//! Oracle Fusion: Financials > Regulatory Reporting

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_REPORT_STATUSES: &[&str] = &["draft", "generated", "under_review", "approved", "submitted", "accepted", "rejected", "archived"];
const VALID_CATEGORIES: &[&str] = &["statutory", "tax", "regulatory", "compliance", "management"];
const VALID_FREQUENCIES: &[&str] = &["monthly", "quarterly", "semi_annual", "annual", "on_demand"];
const VALID_FORMATS: &[&str] = &["xml", "pdf", "csv", "json", "edi"];
const VALID_FILING_STATUSES: &[&str] = &["upcoming", "overdue", "in_progress", "filed", "no_filing_required"];

pub struct RegulatoryReportingEngine { repository: Arc<dyn RegulatoryReportingRepository> }
impl RegulatoryReportingEngine {
    pub fn new(r: Arc<dyn RegulatoryReportingRepository>) -> Self { Self { repository: r } }

    pub async fn create_template(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, authority: &str, category: &str, frequency: &str, format: &str, rows: serde_json::Value, cols: serde_json::Value, rules: serde_json::Value, cb: Option<Uuid>) -> AtlasResult<RegReportTemplate> {
        if code.is_empty() || name.is_empty() { return Err(AtlasError::ValidationFailed("Code and name required".into())); }
        if authority.is_empty() { return Err(AtlasError::ValidationFailed("Authority required".into())); }
        if !VALID_CATEGORIES.contains(&category) { return Err(AtlasError::ValidationFailed(format!("Invalid category '{}'", category))); }
        if !VALID_FREQUENCIES.contains(&frequency) { return Err(AtlasError::ValidationFailed(format!("Invalid frequency '{}'", frequency))); }
        if !VALID_FORMATS.contains(&format) { return Err(AtlasError::ValidationFailed(format!("Invalid format '{}'", format))); }
        if self.repository.get_template(org_id, code).await?.is_some() { return Err(AtlasError::Conflict(format!("Template '{}' already exists", code))); }
        info!("Creating regulatory template {} for {}", code, authority);
        self.repository.create_template(org_id, code, name, desc, authority, category, frequency, format, rows, cols, rules, cb).await
    }
    pub async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RegReportTemplate>> { self.repository.get_template(org_id, code).await }
    pub async fn list_templates(&self, org_id: Uuid, auth: Option<&str>, cat: Option<&str>) -> AtlasResult<Vec<RegReportTemplate>> { self.repository.list_templates(org_id, auth, cat).await }
    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> { self.repository.get_template(org_id, code).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Template '{}' not found", code)))?; self.repository.delete_template(org_id, code).await }
    pub async fn create_report(&self, org_id: Uuid, template_code: &str, report_number: &str, name: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, cb: Option<Uuid>) -> AtlasResult<RegReport> {
        if template_code.is_empty() || report_number.is_empty() || name.is_empty() { return Err(AtlasError::ValidationFailed("All fields required".into())); }
        if pe < ps { return Err(AtlasError::ValidationFailed("Period end must be after start".into())); }
        let t = self.repository.get_template(org_id, template_code).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Template '{}' not found", template_code)))?;
        if self.repository.get_report_by_number(org_id, report_number).await?.is_some() { return Err(AtlasError::Conflict(format!("Report '{}' exists", report_number))); }
        self.repository.create_report(org_id, t.id, Some(template_code), report_number, name, ps, pe, &t.authority, &t.output_format, cb).await
    }
    pub async fn get_report(&self, id: Uuid) -> AtlasResult<Option<RegReport>> { self.repository.get_report(id).await }
    pub async fn list_reports(&self, org_id: Uuid, status: Option<&str>, auth: Option<&str>) -> AtlasResult<Vec<RegReport>> {
        if let Some(s) = status { if !VALID_REPORT_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s))); } }
        self.repository.list_reports(org_id, status, auth).await
    }
    pub async fn submit_for_review(&self, rid: Uuid, rv: Uuid) -> AtlasResult<RegReport> {
        let r = self.repository.get_report(rid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Report {} not found", rid)))?;
        if r.status != "generated" { return Err(AtlasError::WorkflowError(format!("Cannot review in '{}' status", r.status))); }
        self.repository.update_report_status(rid, "under_review", Some(rv), None, None, None, None).await
    }
    pub async fn approve_report(&self, rid: Uuid, ap: Uuid) -> AtlasResult<RegReport> {
        let r = self.repository.get_report(rid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Report {} not found", rid)))?;
        if r.status != "under_review" { return Err(AtlasError::WorkflowError("Must be under_review".into())); }
        self.repository.update_report_status(rid, "approved", None, Some(ap), None, None, None).await
    }
    pub async fn reject_report(&self, rid: Uuid, reason: &str) -> AtlasResult<RegReport> {
        let r = self.repository.get_report(rid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Report {} not found", rid)))?;
        if r.status != "under_review" { return Err(AtlasError::WorkflowError("Must be under_review".into())); }
        if reason.is_empty() { return Err(AtlasError::ValidationFailed("Rejection reason required".into())); }
        self.repository.update_report_status(rid, "rejected", None, None, None, None, Some(reason)).await
    }
    pub async fn submit_report(&self, rid: Uuid, sb: Uuid, fr: Option<&str>) -> AtlasResult<RegReport> {
        let r = self.repository.get_report(rid).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Report {} not found", rid)))?;
        if r.status != "approved" { return Err(AtlasError::WorkflowError("Must be approved".into())); }
        self.repository.update_report_status(rid, "submitted", None, None, Some(sb), fr, None).await
    }
    pub async fn add_report_line(&self, org_id: Uuid, rid: Uuid, ln: i32, rc: &str, rl: &str, cc: &str, cl: &str, amt: &str, desc: Option<&str>, ar: Option<&str>, sub: bool, tot: bool, ind: i32) -> AtlasResult<RegReportLine> {
        if rc.is_empty() || rl.is_empty() { return Err(AtlasError::ValidationFailed("Row code and label required".into())); }
        let a: f64 = amt.parse().map_err(|_| AtlasError::ValidationFailed("Invalid amount".into()))?;
        if a < 0.0 { return Err(AtlasError::ValidationFailed("Amount must be non-negative".into())); }
        self.repository.create_report_line(org_id, rid, ln, rc, rl, cc, cl, amt, desc, ar, sub, tot, ind).await
    }
    pub async fn list_report_lines(&self, rid: Uuid) -> AtlasResult<Vec<RegReportLine>> { self.repository.list_report_lines(rid).await }
    pub async fn create_filing(&self, org_id: Uuid, tc: Option<&str>, auth: &str, rn: &str, freq: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, dd: chrono::NaiveDate, at: Option<Uuid>) -> AtlasResult<RegFilingEntry> {
        if rn.is_empty() || auth.is_empty() { return Err(AtlasError::ValidationFailed("Report name and authority required".into())); }
        if !VALID_FREQUENCIES.contains(&freq) { return Err(AtlasError::ValidationFailed(format!("Invalid frequency '{}'", freq))); }
        if dd < pe { return Err(AtlasError::ValidationFailed("Due date must be on/after period end".into())); }
        let tid = if let Some(c) = tc { self.repository.get_template(org_id, c).await?.map(|t| t.id) } else { None };
        self.repository.create_filing(org_id, tid, tc, auth, rn, freq, ps, pe, dd, at).await
    }
    pub async fn list_filings(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RegFilingEntry>> {
        if let Some(s) = status { if !VALID_FILING_STATUSES.contains(&s) { return Err(AtlasError::ValidationFailed(format!("Invalid status '{}'", s))); } }
        self.repository.list_filings(org_id, status).await
    }
    pub async fn mark_filed(&self, fid: Uuid, rid: Uuid, fb: Uuid, fr: Option<&str>) -> AtlasResult<RegFilingEntry> { self.repository.update_filing_status(fid, "filed", Some(rid), Some(fb), fr).await }
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RegReportingDashboard> { self.repository.get_dashboard(org_id).await }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock { templates: std::sync::Mutex<Vec<RegReportTemplate>> }
    impl Mock { fn new() -> Self { Mock { templates: std::sync::Mutex::new(vec![]) } } }
    #[async_trait]
    impl RegulatoryReportingRepository for Mock {
        async fn create_template(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, auth: &str, cat: &str, freq: &str, fmt: &str, rows: serde_json::Value, cols: serde_json::Value, rules: serde_json::Value, cb: Option<Uuid>) -> AtlasResult<RegReportTemplate> {
            let t = RegReportTemplate { id: Uuid::new_v4(), organization_id: org_id, code: code.into(), name: name.into(), description: desc.map(Into::into), authority: auth.into(), report_category: cat.into(), filing_frequency: freq.into(), output_format: fmt.into(), row_definitions: rows, column_definitions: cols, validation_rules: rules, is_active: true, metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() };
            self.templates.lock().unwrap().push(t.clone());
            Ok(t)
        }
        async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RegReportTemplate>> { Ok(self.templates.lock().unwrap().iter().find(|t| t.organization_id == org_id && t.code == code).cloned()) }
        async fn get_template_by_id(&self, _: Uuid) -> AtlasResult<Option<RegReportTemplate>> { Ok(None) }
        async fn list_templates(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<RegReportTemplate>> { Ok(vec![]) }
        async fn delete_template(&self, _: Uuid, _: &str) -> AtlasResult<()> { Ok(()) }
        async fn create_report(&self, org_id: Uuid, tid: Uuid, tc: Option<&str>, rn: &str, name: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, auth: &str, fmt: &str, cb: Option<Uuid>) -> AtlasResult<RegReport> {
            Ok(RegReport { id: Uuid::new_v4(), organization_id: org_id, template_id: tid, template_code: tc.map(Into::into), report_number: rn.into(), name: name.into(), status: "draft".into(), period_start: ps, period_end: pe, authority: auth.into(), output_format: fmt.into(), total_debits: "0".into(), total_credits: "0".into(), line_count: 0, generated_at: None, reviewed_by: None, reviewed_at: None, approved_by: None, approved_at: None, submitted_by: None, submitted_at: None, filing_reference: None, rejection_reason: None, metadata: serde_json::json!({}), created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn get_report(&self, _: Uuid) -> AtlasResult<Option<RegReport>> { Ok(None) }
        async fn get_report_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<RegReport>> { Ok(None) }
        async fn list_reports(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<RegReport>> { Ok(vec![]) }
        async fn update_report_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<Uuid>, _: Option<Uuid>, _: Option<&str>, _: Option<&str>) -> AtlasResult<RegReport> { Err(AtlasError::EntityNotFound("M".into())) }
        async fn update_report_line_count(&self, _: Uuid, _: i32, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
        async fn create_report_line(&self, org_id: Uuid, rid: Uuid, ln: i32, rc: &str, rl: &str, cc: &str, cl: &str, amt: &str, desc: Option<&str>, ar: Option<&str>, sub: bool, tot: bool, ind: i32) -> AtlasResult<RegReportLine> {
            Ok(RegReportLine { id: Uuid::new_v4(), organization_id: org_id, report_id: rid, line_number: ln, row_code: rc.into(), row_label: rl.into(), column_code: cc.into(), column_label: cl.into(), amount: amt.into(), description: desc.map(Into::into), account_range: ar.map(Into::into), is_subtotal: sub, is_total: tot, indent_level: ind, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn list_report_lines(&self, _: Uuid) -> AtlasResult<Vec<RegReportLine>> { Ok(vec![]) }
        async fn create_filing(&self, org_id: Uuid, tid: Option<Uuid>, tc: Option<&str>, auth: &str, rn: &str, freq: &str, ps: chrono::NaiveDate, pe: chrono::NaiveDate, dd: chrono::NaiveDate, at: Option<Uuid>) -> AtlasResult<RegFilingEntry> {
            Ok(RegFilingEntry { id: Uuid::new_v4(), organization_id: org_id, template_id: tid, template_code: tc.map(Into::into), authority: auth.into(), report_name: rn.into(), filing_frequency: freq.into(), period_start: ps, period_end: pe, due_date: dd, status: "upcoming".into(), assigned_to: at, report_id: None, filed_at: None, filed_by: None, filing_reference: None, notes: None, metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now() })
        }
        async fn list_filings(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<RegFilingEntry>> { Ok(vec![]) }
        async fn update_filing_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<RegFilingEntry> { Err(AtlasError::EntityNotFound("M".into())) }
        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<RegReportingDashboard> { Ok(RegReportingDashboard { total_templates: 0, active_templates: 0, total_reports: 0, draft_reports: 0, pending_review: 0, pending_submission: 0, submitted_reports: 0, overdue_filings: 0, upcoming_filings: 0, filings_by_authority: serde_json::json!([]) }) }
    }
    fn eng() -> RegulatoryReportingEngine { RegulatoryReportingEngine::new(Arc::new(Mock::new())) }

    #[test]
    fn test_valid_constants() { assert_eq!(VALID_REPORT_STATUSES.len(), 8); assert_eq!(VALID_CATEGORIES.len(), 5); assert_eq!(VALID_FREQUENCIES.len(), 5); assert_eq!(VALID_FORMATS.len(), 5); assert_eq!(VALID_FILING_STATUSES.len(), 5); }

    #[tokio::test]
    async fn test_create_template_valid() { let t = eng().create_template(Uuid::new_v4(), "SEC-10K", "Annual", None, "SEC", "statutory", "annual", "xml", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await.unwrap(); assert_eq!(t.code, "SEC-10K"); assert!(t.is_active); }

    #[tokio::test]
    async fn test_create_template_empty_code() { assert!(eng().create_template(Uuid::new_v4(), "", "R", None, "SEC", "statutory", "annual", "xml", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_template_invalid_category() { assert!(eng().create_template(Uuid::new_v4(), "T", "R", None, "SEC", "bad", "annual", "xml", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_template_invalid_frequency() { assert!(eng().create_template(Uuid::new_v4(), "T", "R", None, "SEC", "statutory", "bad", "xml", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_template_invalid_format() { assert!(eng().create_template(Uuid::new_v4(), "T", "R", None, "SEC", "statutory", "annual", "docx", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_template_duplicate() { let org = Uuid::new_v4(); let e = eng(); let _ = e.create_template(org, "DUP", "R1", None, "SEC", "statutory", "annual", "xml", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await; assert!(e.create_template(org, "DUP", "R2", None, "SEC", "tax", "quarterly", "pdf", serde_json::json!([]), serde_json::json!([]), serde_json::json!([]), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_report_template_not_found() { assert!(eng().create_report(Uuid::new_v4(), "NOPE", "R-1", "R", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,3,31).unwrap(), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_report_end_before_start() { assert!(eng().create_report(Uuid::new_v4(), "T", "R-1", "R", chrono::NaiveDate::from_ymd_opt(2026,6,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), None).await.is_err()); }

    #[tokio::test]
    async fn test_list_reports_invalid_status() { assert!(eng().list_reports(Uuid::new_v4(), Some("bad"), None).await.is_err()); }
    #[tokio::test]
    async fn test_list_reports_valid() { assert!(eng().list_reports(Uuid::new_v4(), Some("draft"), None).await.unwrap().is_empty()); }

    #[tokio::test]
    async fn test_create_filing_invalid_freq() { assert!(eng().create_filing(Uuid::new_v4(), None, "SEC", "10-K", "biennial", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,12,31).unwrap(), chrono::NaiveDate::from_ymd_opt(2027,3,31).unwrap(), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_filing_due_before_end() { assert!(eng().create_filing(Uuid::new_v4(), None, "SEC", "10-K", "annual", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,12,31).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,6,30).unwrap(), None).await.is_err()); }

    #[tokio::test]
    async fn test_create_filing_valid() { let f = eng().create_filing(Uuid::new_v4(), None, "SEC", "10-K", "annual", chrono::NaiveDate::from_ymd_opt(2026,1,1).unwrap(), chrono::NaiveDate::from_ymd_opt(2026,12,31).unwrap(), chrono::NaiveDate::from_ymd_opt(2027,3,31).unwrap(), None).await.unwrap(); assert_eq!(f.authority, "SEC"); assert_eq!(f.status, "upcoming"); }

    #[tokio::test]
    async fn test_list_filings_invalid_status() { assert!(eng().list_filings(Uuid::new_v4(), Some("bad")).await.is_err()); }

    #[tokio::test]
    async fn test_add_line_empty_row_code() { assert!(eng().add_report_line(Uuid::new_v4(), Uuid::new_v4(), 1, "", "Rev", "C1", "CP", "100000.00", None, None, false, false, 0).await.is_err()); }

    #[tokio::test]
    async fn test_add_line_negative_amount() { assert!(eng().add_report_line(Uuid::new_v4(), Uuid::new_v4(), 1, "R", "Rev", "C1", "CP", "-500.00", None, None, false, false, 0).await.is_err()); }
}
