//! Tax Reporting Engine
//!
//! Core tax reporting operations:
//! - Tax return template management (VAT, GST, income tax forms)
//! - Template line definitions (boxes/fields on tax returns)
//! - Tax return preparation and workflow (draft -> submitted -> filed -> paid)
//! - Filing calendar management
//! - Dashboard summary
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Tax > Tax Reporting

use atlas_shared::{
    TaxReturnTemplate, TaxReturnTemplateLine, TaxReturn, TaxReturnLine,
    TaxFilingCalendarEntry, TaxReportingDashboardSummary,
    AtlasError, AtlasResult,
};
use super::TaxReportingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid tax types
#[allow(dead_code)]
const VALID_TAX_TYPES: &[&str] = &[
    "vat", "gst", "sales_tax", "corporate_income", "withholding",
];

/// Valid filing frequencies
#[allow(dead_code)]
const VALID_FILING_FREQUENCIES: &[&str] = &[
    "monthly", "quarterly", "semi_annual", "annual",
];

/// Valid line types
#[allow(dead_code)]
const VALID_LINE_TYPES: &[&str] = &[
    "input", "calculated", "total", "informational",
];

/// Valid return statuses
#[allow(dead_code)]
const VALID_RETURN_STATUSES: &[&str] = &[
    "draft", "submitted", "filed", "paid", "amended", "rejected",
];

/// Valid filing statuses
#[allow(dead_code)]
const VALID_FILING_STATUSES: &[&str] = &[
    "upcoming", "due_soon", "overdue", "filed", "extended",
];

/// Valid filing methods
#[allow(dead_code)]
const VALID_FILING_METHODS: &[&str] = &["electronic", "paper"];

/// Tax Reporting Engine
pub struct TaxReportingEngine {
    repository: Arc<dyn TaxReportingRepository>,
}

impl TaxReportingEngine {
    pub fn new(repository: Arc<dyn TaxReportingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Template Management
    // ========================================================================

    /// Create a tax return template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        jurisdiction_code: Option<&str>,
        filing_frequency: &str,
        return_form_number: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturnTemplate> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template code and name are required".to_string(),
            ));
        }
        if !VALID_TAX_TYPES.contains(&tax_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tax_type '{}'. Must be one of: {}", tax_type, VALID_TAX_TYPES.join(", ")
            )));
        }
        if !VALID_FILING_FREQUENCIES.contains(&filing_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid filing_frequency '{}'. Must be one of: {}", filing_frequency, VALID_FILING_FREQUENCIES.join(", ")
            )));
        }

        if self.repository.get_template_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("Template code '{}' already exists", code)
            ));
        }

        info!("Creating tax return template '{}' for {}", code, tax_type);

        self.repository.create_template(
            org_id, code, name, description, tax_type,
            jurisdiction_code, filing_frequency, return_form_number,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get template by ID
    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<TaxReturnTemplate>> {
        self.repository.get_template(id).await
    }

    /// Get template by code
    pub async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxReturnTemplate>> {
        self.repository.get_template_by_code(org_id, code).await
    }

    /// List templates
    pub async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<TaxReturnTemplate>> {
        self.repository.list_templates(org_id).await
    }

    // ========================================================================
    // Template Lines
    // ========================================================================

    /// Add a line to a template
    pub async fn add_template_line(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        box_code: &str,
        box_name: &str,
        description: Option<&str>,
        line_type: &str,
        calculation_formula: Option<&str>,
        account_code_filter: Option<&str>,
        tax_rate_code_filter: Option<&str>,
        is_debit: bool,
        display_order: i32,
    ) -> AtlasResult<TaxReturnTemplateLine> {
        let _template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template {} not found", template_id)
            ))?;

        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}", line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        let lines = self.repository.list_template_lines(template_id).await?;
        let line_number = (lines.len() as i32) + 1;

        info!("Adding template line {} ({}) to template", box_code, box_name);

        self.repository.create_template_line(
            org_id, template_id, line_number, box_code, box_name,
            description, line_type, calculation_formula,
            account_code_filter, tax_rate_code_filter, is_debit, display_order,
        ).await
    }

    /// List template lines
    pub async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<TaxReturnTemplateLine>> {
        self.repository.list_template_lines(template_id).await
    }

    // ========================================================================
    // Tax Returns
    // ========================================================================

    /// Create a tax return from a template
    pub async fn create_return(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        filing_period_start: chrono::NaiveDate,
        filing_period_end: chrono::NaiveDate,
        filing_due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn> {
        let template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template {} not found", template_id)
            ))?;

        if !template.is_active {
            return Err(AtlasError::ValidationFailed(
                "Cannot create return from inactive template".to_string(),
            ));
        }

        if filing_period_start >= filing_period_end {
            return Err(AtlasError::ValidationFailed(
                "Filing period start must be before end".to_string(),
            ));
        }

        let return_number = format!("TXR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating tax return {} for period {} to {}",
            return_number, filing_period_start, filing_period_end);

        let tax_return = self.repository.create_return(
            org_id, &return_number, template_id,
            Some(&template.name), Some(&template.tax_type),
            template.jurisdiction_code.as_deref(),
            filing_period_start, filing_period_end, filing_due_date,
            created_by,
        ).await?;

        // Pre-populate lines from template
        let template_lines = self.repository.list_template_lines(template_id).await?;
        for tl in &template_lines {
            self.repository.create_return_line(
                org_id, tax_return.id, Some(tl.id), tl.line_number,
                &tl.box_code, Some(&tl.box_name), &tl.line_type,
                "0", "0", None, "0", None, 0,
            ).await?;
        }

        Ok(tax_return)
    }

    /// Get tax return by ID
    pub async fn get_return(&self, id: Uuid) -> AtlasResult<Option<TaxReturn>> {
        self.repository.get_return(id).await
    }

    /// List tax returns
    pub async fn list_returns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TaxReturn>> {
        if let Some(s) = status {
            if !VALID_RETURN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RETURN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_returns(org_id, status).await
    }

    /// List return lines
    pub async fn list_return_lines(&self, tax_return_id: Uuid) -> AtlasResult<Vec<TaxReturnLine>> {
        self.repository.list_return_lines(tax_return_id).await
    }

    /// Update a return line amount
    pub async fn update_return_line(
        &self,
        line_id: Uuid,
        amount: &str,
        override_amount: Option<&str>,
    ) -> AtlasResult<TaxReturnLine> {
        let _line = self.repository.get_return_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Return line {} not found", line_id)))?;

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        let final_amount = if let Some(override_amt) = override_amount {
            let oamt: f64 = override_amt.parse().map_err(|_| AtlasError::ValidationFailed(
                "Override amount must be a valid number".to_string(),
            ))?;
            oamt
        } else {
            amt
        };

        self.repository.update_return_line(
            line_id, amount, override_amount, &format!("{:.2}", final_amount),
        ).await
    }

    // ========================================================================
    // Return Workflow
    // ========================================================================

    /// Submit return for review
    pub async fn submit_return(&self, return_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<TaxReturn> {
        let tax_return = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Return {} not found", return_id)))?;

        if tax_return.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit return in '{}' status. Must be 'draft'.", tax_return.status)
            ));
        }

        // Calculate totals from lines
        let lines = self.repository.list_return_lines(return_id).await?;
        let total_tax: f64 = lines.iter()
            .filter(|l| l.line_type != "informational")
            .map(|l| l.final_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let output_tax: f64 = lines.iter()
            .filter(|l| l.line_type != "informational")
            .filter(|l| l.box_code.starts_with('O') || l.box_code.contains("output"))
            .map(|l| l.final_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let input_tax: f64 = lines.iter()
            .filter(|l| l.line_type != "informational")
            .filter(|l| l.box_code.starts_with('I') || l.box_code.contains("input"))
            .map(|l| l.final_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let net_due = output_tax - input_tax;

        self.repository.update_return_totals(
            return_id,
            &format!("{:.2}", total_tax),
            &format!("{:.2}", output_tax), // total_taxable (approx)
            "0", // exempt
            &format!("{:.2}", input_tax),
            &format!("{:.2}", output_tax),
            &format!("{:.2}", net_due),
            &format!("{:.2}", net_due), // total_amount_due
        ).await?;

        info!("Submitting tax return {} (net due: {:.2})", tax_return.return_number, net_due);
        self.repository.update_return_status(return_id, "submitted", submitted_by, None, None).await
    }

    /// File a submitted return
    pub async fn file_return(
        &self,
        return_id: Uuid,
        filing_method: &str,
        filing_reference: Option<&str>,
        filed_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn> {
        let tax_return = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Return {} not found", return_id)))?;

        if tax_return.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot file return in '{}' status. Must be 'submitted'.", tax_return.status)
            ));
        }

        if !VALID_FILING_METHODS.contains(&filing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid filing_method '{}'. Must be one of: {}", filing_method, VALID_FILING_METHODS.join(", ")
            )));
        }

        info!("Filing tax return {} via {}", tax_return.return_number, filing_method);
        self.repository.update_return_filing(
            return_id, "filed", filing_method, filing_reference, filed_by,
        ).await
    }

    /// Mark return as paid
    pub async fn mark_paid(
        &self,
        return_id: Uuid,
        payment_amount: &str,
        payment_reference: Option<&str>,
    ) -> AtlasResult<TaxReturn> {
        let tax_return = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Return {} not found", return_id)))?;

        if tax_return.status != "filed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot mark paid from '{}' status. Must be 'filed'.", tax_return.status)
            ));
        }

        let amt: f64 = payment_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Payment amount must be a valid number".to_string(),
        ))?;
        if amt < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Payment amount must be non-negative".to_string(),
            ));
        }

        info!("Marking tax return {} as paid ({:.2})", tax_return.return_number, amt);
        self.repository.update_return_payment(
            return_id, "paid", payment_amount, payment_reference,
        ).await
    }

    // ========================================================================
    // Filing Calendar
    // ========================================================================

    /// List upcoming filing calendar entries
    pub async fn list_filing_calendar(&self, org_id: Uuid) -> AtlasResult<Vec<TaxFilingCalendarEntry>> {
        self.repository.list_filing_calendar(org_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get tax reporting dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TaxReportingDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    /// Calculate net tax due (output - input)
    pub fn calculate_net_tax(output_tax: f64, input_tax: f64) -> f64 {
        output_tax - input_tax
    }

    /// Check if a filing is overdue
    pub fn is_overdue(due_date: chrono::NaiveDate, filed: bool) -> bool {
        if filed {
            return false;
        }
        let today = chrono::Utc::now().date_naive();
        due_date < today
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tax_types() {
        assert!(VALID_TAX_TYPES.contains(&"vat"));
        assert!(VALID_TAX_TYPES.contains(&"gst"));
        assert!(VALID_TAX_TYPES.contains(&"sales_tax"));
        assert!(VALID_TAX_TYPES.contains(&"corporate_income"));
        assert!(VALID_TAX_TYPES.contains(&"withholding"));
        assert_eq!(VALID_TAX_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_filing_frequencies() {
        assert!(VALID_FILING_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_FILING_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_FILING_FREQUENCIES.contains(&"semi_annual"));
        assert!(VALID_FILING_FREQUENCIES.contains(&"annual"));
        assert_eq!(VALID_FILING_FREQUENCIES.len(), 4);
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"input"));
        assert!(VALID_LINE_TYPES.contains(&"calculated"));
        assert!(VALID_LINE_TYPES.contains(&"total"));
        assert!(VALID_LINE_TYPES.contains(&"informational"));
        assert_eq!(VALID_LINE_TYPES.len(), 4);
    }

    #[test]
    fn test_valid_return_statuses() {
        assert!(VALID_RETURN_STATUSES.contains(&"draft"));
        assert!(VALID_RETURN_STATUSES.contains(&"submitted"));
        assert!(VALID_RETURN_STATUSES.contains(&"filed"));
        assert!(VALID_RETURN_STATUSES.contains(&"paid"));
        assert!(VALID_RETURN_STATUSES.contains(&"amended"));
        assert!(VALID_RETURN_STATUSES.contains(&"rejected"));
        assert_eq!(VALID_RETURN_STATUSES.len(), 6);
    }

    #[test]
    fn test_valid_filing_statuses() {
        assert!(VALID_FILING_STATUSES.contains(&"upcoming"));
        assert!(VALID_FILING_STATUSES.contains(&"due_soon"));
        assert!(VALID_FILING_STATUSES.contains(&"overdue"));
        assert!(VALID_FILING_STATUSES.contains(&"filed"));
        assert!(VALID_FILING_STATUSES.contains(&"extended"));
        assert_eq!(VALID_FILING_STATUSES.len(), 5);
    }

    #[test]
    fn test_valid_filing_methods() {
        assert!(VALID_FILING_METHODS.contains(&"electronic"));
        assert!(VALID_FILING_METHODS.contains(&"paper"));
        assert_eq!(VALID_FILING_METHODS.len(), 2);
    }

    #[test]
    fn test_calculate_net_tax_owed() {
        // Output > Input: net tax owed
        let net = TaxReportingEngine::calculate_net_tax(15000.0, 8000.0);
        assert!((net - 7000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_net_tax_refund() {
        // Input > Output: refund due
        let net = TaxReportingEngine::calculate_net_tax(5000.0, 8000.0);
        assert!((net - (-3000.0)).abs() < 0.01);
    }

    #[test]
    fn test_calculate_net_tax_zero() {
        let net = TaxReportingEngine::calculate_net_tax(10000.0, 10000.0);
        assert!(net.abs() < 0.01);
    }

    #[test]
    fn test_is_overdue_future_date() {
        let future = chrono::NaiveDate::from_ymd_opt(2030, 12, 31).unwrap();
        assert!(!TaxReportingEngine::is_overdue(future, false));
    }

    #[test]
    fn test_is_overdue_past_date_not_filed() {
        let past = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        assert!(TaxReportingEngine::is_overdue(past, false));
    }

    #[test]
    fn test_is_overdue_past_date_filed() {
        let past = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        assert!(!TaxReportingEngine::is_overdue(past, true));
    }
}
