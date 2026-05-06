//! Customer Statement Engine
//!
//! Manages customer account statement lifecycle: creation with balance
//! calculation, line aggregation, aging breakdown, delivery tracking,
//! and statement generation with forward balance from prior periods.
//!
//! Oracle Fusion Cloud ERP equivalent: Receivables > Billing > Balance Forward Billing

use atlas_shared::{
    CustomerStatement, CustomerStatementLine, CustomerStatementSummary,
    AtlasError, AtlasResult,
};
use super::CustomerStatementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid billing cycles
const VALID_BILLING_CYCLES: &[&str] = &["monthly", "quarterly", "weekly", "custom"];

/// Valid statement statuses
const VALID_STATUSES: &[&str] = &[
    "draft", "generated", "sent", "viewed", "archived", "cancelled",
];

/// Valid line types
pub const VALID_LINE_TYPES: &[&str] = &[
    "opening_balance", "invoice", "payment", "credit_memo",
    "debit_memo", "adjustment", "finance_charge", "closing_balance",
];

/// Valid delivery methods
const VALID_DELIVERY_METHODS: &[&str] = &["email", "print", "xml", "edi"];

/// Calculate the closing balance from opening balance and activity.
/// closing = opening + charges - payments - credits + adjustments
pub fn calculate_closing_balance(
    opening_balance: f64,
    total_charges: f64,
    total_payments: f64,
    total_credits: f64,
    total_adjustments: f64,
) -> f64 {
    opening_balance + total_charges - total_payments - total_credits + total_adjustments
}

/// Calculate the amount due (positive closing balance, or zero).
pub fn calculate_amount_due(closing_balance: f64) -> f64 {
    closing_balance.max(0.0)
}

/// Compute running balances for a sorted list of statement lines.
/// The running balance starts from the opening balance and is updated
/// by each line based on its type:
///   - invoice, debit_memo, finance_charge: increase balance
///   - payment, credit_memo: decrease balance
///   - adjustment: increase balance (positive) or decrease (negative)
///   - opening_balance, closing_balance: no change to running balance
pub fn compute_running_balances(
    lines: &mut [(String, f64, Option<f64>)], // (line_type, amount, running_balance)
    opening_balance: f64,
) {
    let mut running = opening_balance;
    for (line_type, amount, running_balance_slot) in lines.iter_mut() {
        match line_type.as_str() {
            "invoice" | "debit_memo" | "finance_charge" => {
                running += *amount;
            }
            "payment" | "credit_memo" => {
                running -= *amount;
            }
            "adjustment" => {
                running += *amount; // positive adjustment increases, negative decreases
            }
            "opening_balance" | "closing_balance" => {
                // Don't change running balance
            }
            _ => {}
        }
        *running_balance_slot = Some(running);
    }
}

/// Compute aging breakdown from a reference date and a list of (due_date, amount) pairs.
/// Returns (current, aging_1_30, aging_31_60, aging_61_90, aging_91_120, aging_121_plus).
pub fn compute_aging_breakdown(
    reference_date: chrono::NaiveDate,
    items: &[(chrono::NaiveDate, f64)], // (due_date, outstanding_amount)
) -> (f64, f64, f64, f64, f64, f64) {
    let mut aging_current = 0.0_f64;
    let mut aging_1_30 = 0.0;
    let mut aging_31_60 = 0.0;
    let mut aging_61_90 = 0.0;
    let mut aging_91_120 = 0.0;
    let mut aging_121_plus = 0.0;

    for (due_date, amount) in items {
        if *amount <= 0.0 {
            continue;
        }
        let days_overdue = (reference_date - *due_date).num_days();
        if days_overdue <= 0 {
            aging_current += amount;
        } else if days_overdue <= 30 {
            aging_1_30 += amount;
        } else if days_overdue <= 60 {
            aging_31_60 += amount;
        } else if days_overdue <= 90 {
            aging_61_90 += amount;
        } else if days_overdue <= 120 {
            aging_91_120 += amount;
        } else {
            aging_121_plus += amount;
        }
    }

    (aging_current, aging_1_30, aging_31_60, aging_61_90, aging_91_120, aging_121_plus)
}

/// Customer Statement Engine
pub struct CustomerStatementEngine {
    repository: Arc<dyn CustomerStatementRepository>,
}

impl CustomerStatementEngine {
    pub fn new(repository: Arc<dyn CustomerStatementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Statement Creation
    // ========================================================================

    /// Create a new customer statement
    pub async fn create_statement(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        statement_date: chrono::NaiveDate,
        billing_period_from: chrono::NaiveDate,
        billing_period_to: chrono::NaiveDate,
        billing_cycle: &str,
        opening_balance: &str,
        total_charges: &str,
        total_payments: &str,
        total_credits: &str,
        total_adjustments: &str,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        currency_code: &str,
        delivery_method: Option<&str>,
        delivery_email: Option<&str>,
        previous_statement_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CustomerStatement> {
        if !VALID_BILLING_CYCLES.contains(&billing_cycle) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing cycle '{}'. Must be one of: {}",
                billing_cycle, VALID_BILLING_CYCLES.join(", ")
            )));
        }
        if billing_period_from >= billing_period_to {
            return Err(AtlasError::ValidationFailed(
                "Billing period 'from' date must be before 'to' date".to_string(),
            ));
        }
        if let Some(dm) = delivery_method {
            if !VALID_DELIVERY_METHODS.contains(&dm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid delivery method '{}'. Must be one of: {}",
                    dm, VALID_DELIVERY_METHODS.join(", ")
                )));
            }
        }

        let ob: f64 = opening_balance.parse().map_err(|_| AtlasError::ValidationFailed(
            "Opening balance must be a valid number".to_string(),
        ))?;
        let charges: f64 = total_charges.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total charges must be a valid number".to_string(),
        ))?;
        let payments: f64 = total_payments.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total payments must be a valid number".to_string(),
        ))?;
        let credits: f64 = total_credits.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total credits must be a valid number".to_string(),
        ))?;
        let adjustments: f64 = total_adjustments.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total adjustments must be a valid number".to_string(),
        ))?;

        let closing = calculate_closing_balance(ob, charges, payments, credits, adjustments);
        let amount_due = calculate_amount_due(closing);

        // Generate statement number
        let next_num = self.repository.get_next_statement_number(org_id).await?;
        let statement_number = format!("CS-{:05}", next_num);

        info!(
            "Creating customer statement {} for customer {} in org {}",
            statement_number, customer_id, org_id
        );

        self.repository.create_statement(
            org_id, &statement_number, customer_id, customer_number, customer_name,
            statement_date, billing_period_from, billing_period_to, billing_cycle,
            opening_balance, total_charges, total_payments, total_credits, total_adjustments,
            &format!("{:.2}", closing),
            &format!("{:.2}", amount_due),
            aging_current, aging_1_30, aging_31_60, aging_61_90, aging_91_120, aging_121_plus,
            currency_code, delivery_method, delivery_email,
            previous_statement_id, notes, created_by,
        ).await
    }

    /// Get a statement by ID
    pub async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CustomerStatement>> {
        self.repository.get_statement(id).await
    }

    /// Get a statement by number
    pub async fn get_statement_by_number(
        &self,
        org_id: Uuid,
        statement_number: &str,
    ) -> AtlasResult<Option<CustomerStatement>> {
        self.repository.get_statement_by_number(org_id, statement_number).await
    }

    /// List statements with optional filters
    pub async fn list_statements(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
        billing_cycle: Option<&str>,
    ) -> AtlasResult<Vec<CustomerStatement>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_STATUSES.join(", ")
                )));
            }
        }
        if let Some(bc) = billing_cycle {
            if !VALID_BILLING_CYCLES.contains(&bc) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid billing cycle '{}'. Must be one of: {}",
                    bc, VALID_BILLING_CYCLES.join(", ")
                )));
            }
        }
        self.repository.list_statements(org_id, customer_id, status, billing_cycle).await
    }

    // ========================================================================
    // Statement Lifecycle
    // ========================================================================

    /// Generate a statement (transition from draft to generated)
    pub async fn generate_statement(&self, id: Uuid) -> AtlasResult<CustomerStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Customer statement {} not found", id)
            ))?;

        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot generate statement in '{}' status. Must be 'draft'.",
                stmt.status
            )));
        }

        info!("Generating customer statement {}", stmt.statement_number);
        self.repository.update_statement_status(id, "generated").await
    }

    /// Mark a statement as sent
    pub async fn send_statement(&self, id: Uuid) -> AtlasResult<CustomerStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Customer statement {} not found", id)
            ))?;

        if stmt.status != "generated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot send statement in '{}' status. Must be 'generated'.",
                stmt.status
            )));
        }

        info!("Sending customer statement {}", stmt.statement_number);
        self.repository.update_statement_status(id, "sent").await
    }

    /// Mark a statement as viewed
    pub async fn mark_viewed(&self, id: Uuid) -> AtlasResult<CustomerStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Customer statement {} not found", id)
            ))?;

        if stmt.status != "sent" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot mark as viewed statement in '{}' status. Must be 'sent'.",
                stmt.status
            )));
        }

        info!("Marking customer statement {} as viewed", stmt.statement_number);
        self.repository.update_statement_status(id, "viewed").await
    }

    /// Archive a statement
    pub async fn archive_statement(&self, id: Uuid) -> AtlasResult<CustomerStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Customer statement {} not found", id)
            ))?;

        if stmt.status != "viewed" && stmt.status != "sent" && stmt.status != "generated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot archive statement in '{}' status.",
                stmt.status
            )));
        }

        info!("Archiving customer statement {}", stmt.statement_number);
        self.repository.update_statement_status(id, "archived").await
    }

    /// Cancel a statement (only if draft)
    pub async fn cancel_statement(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<CustomerStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Customer statement {} not found", id)
            ))?;

        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel statement in '{}' status. Must be 'draft'.",
                stmt.status
            )));
        }

        info!("Cancelling customer statement {}", stmt.statement_number);

        if let Some(r) = reason {
            self.repository.update_statement_notes(id, Some(r)).await?;
        }

        self.repository.update_statement_status(id, "cancelled").await
    }

    /// Resend a statement (transition from sent/viewed back to sent, updating sent_at)
    pub async fn resend_statement(&self, id: Uuid) -> AtlasResult<CustomerStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Customer statement {} not found", id)
            ))?;

        if stmt.status != "sent" && stmt.status != "viewed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot resend statement in '{}' status. Must be 'sent' or 'viewed'.",
                stmt.status
            )));
        }

        info!("Resending customer statement {}", stmt.statement_number);
        self.repository.update_statement_status(id, "sent").await
    }

    // ========================================================================
    // Statement Lines
    // ========================================================================

    /// Add a line to a statement
    pub async fn add_statement_line(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        line_type: &str,
        transaction_id: Option<Uuid>,
        transaction_number: Option<&str>,
        transaction_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        original_amount: Option<&str>,
        amount: &str,
        description: Option<&str>,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> AtlasResult<CustomerStatementLine> {
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        let stmt = self.repository.get_statement(statement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Statement {} not found", statement_id)
            ))?;

        if stmt.organization_id != org_id {
            return Err(AtlasError::Forbidden("Not authorized".to_string()));
        }

        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add lines to a statement that is not in 'draft' status".to_string(),
            ));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        let orig_amt: Option<f64> = match original_amount {
            Some(s) => Some(s.parse().map_err(|_| AtlasError::ValidationFailed(
                "Original amount must be a valid number".to_string(),
            ))?),
            None => None,
        };

        // Get next display order
        let next_order = self.repository.get_next_line_order(statement_id).await?;

        info!(
            "Adding {} line to statement {} (amount: {})",
            line_type, stmt.statement_number, amount
        );

        self.repository.create_statement_line(
            org_id, statement_id, line_type,
            transaction_id, transaction_number, transaction_date, due_date,
            original_amount.map(|s| format!("{:.2}", orig_amt.unwrap_or(0.0))).as_deref(),
            &format!("{:.2}", amt),
            description, reference_type, reference_id,
            next_order, metadata,
        ).await
    }

    /// List statement lines
    pub async fn list_statement_lines(
        &self,
        statement_id: Uuid,
    ) -> AtlasResult<Vec<CustomerStatementLine>> {
        self.repository.list_statement_lines(statement_id).await
    }

    /// Remove a statement line (only if statement is draft)
    pub async fn remove_statement_line(&self, statement_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        let stmt = self.repository.get_statement(statement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Statement {} not found", statement_id)
            ))?;

        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot remove lines from a statement that is not in 'draft' status".to_string(),
            ));
        }

        self.repository.delete_statement_line(line_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get statement summary dashboard
    pub async fn get_statement_summary(&self, org_id: Uuid) -> AtlasResult<CustomerStatementSummary> {
        self.repository.get_statement_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_billing_cycles() {
        assert!(VALID_BILLING_CYCLES.contains(&"monthly"));
        assert!(VALID_BILLING_CYCLES.contains(&"quarterly"));
        assert!(VALID_BILLING_CYCLES.contains(&"weekly"));
        assert!(VALID_BILLING_CYCLES.contains(&"custom"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"generated"));
        assert!(VALID_STATUSES.contains(&"sent"));
        assert!(VALID_STATUSES.contains(&"viewed"));
        assert!(VALID_STATUSES.contains(&"archived"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"opening_balance"));
        assert!(VALID_LINE_TYPES.contains(&"invoice"));
        assert!(VALID_LINE_TYPES.contains(&"payment"));
        assert!(VALID_LINE_TYPES.contains(&"credit_memo"));
        assert!(VALID_LINE_TYPES.contains(&"debit_memo"));
        assert!(VALID_LINE_TYPES.contains(&"adjustment"));
        assert!(VALID_LINE_TYPES.contains(&"finance_charge"));
        assert!(VALID_LINE_TYPES.contains(&"closing_balance"));
    }

    #[test]
    fn test_valid_delivery_methods() {
        assert!(VALID_DELIVERY_METHODS.contains(&"email"));
        assert!(VALID_DELIVERY_METHODS.contains(&"print"));
        assert!(VALID_DELIVERY_METHODS.contains(&"xml"));
        assert!(VALID_DELIVERY_METHODS.contains(&"edi"));
    }

    #[test]
    fn test_closing_balance_positive() {
        // opening=1000, charges=500, payments=200, credits=50, adjustments=0
        // closing = 1000 + 500 - 200 - 50 + 0 = 1250
        let closing = calculate_closing_balance(1000.0, 500.0, 200.0, 50.0, 0.0);
        assert!((closing - 1250.0).abs() < 0.01);
    }

    #[test]
    fn test_closing_balance_negative() {
        // opening=-200, charges=100, payments=0, credits=0, adjustments=0
        // closing = -200 + 100 = -100
        let closing = calculate_closing_balance(-200.0, 100.0, 0.0, 0.0, 0.0);
        assert!((closing - (-100.0)).abs() < 0.01);
    }

    #[test]
    fn test_closing_balance_with_adjustments() {
        // opening=500, charges=200, payments=100, credits=50, adjustments=25
        // closing = 500 + 200 - 100 - 50 + 25 = 575
        let closing = calculate_closing_balance(500.0, 200.0, 100.0, 50.0, 25.0);
        assert!((closing - 575.0).abs() < 0.01);
    }

    #[test]
    fn test_closing_balance_negative_adjustment() {
        // opening=1000, charges=0, payments=0, credits=0, adjustments=-300
        // closing = 1000 + 0 - 0 - 0 + (-300) = 700
        let closing = calculate_closing_balance(1000.0, 0.0, 0.0, 0.0, -300.0);
        assert!((closing - 700.0).abs() < 0.01);
    }

    #[test]
    fn test_amount_due_positive() {
        assert!((calculate_amount_due(1500.0) - 1500.0).abs() < 0.01);
    }

    #[test]
    fn test_amount_due_negative_is_zero() {
        assert!((calculate_amount_due(-200.0)).abs() < 0.01);
    }

    #[test]
    fn test_amount_due_zero() {
        assert!((calculate_amount_due(0.0)).abs() < 0.01);
    }

    #[test]
    fn test_running_balances_basic() {
        let mut lines: Vec<(String, f64, Option<f64>)> = vec![
            ("invoice".to_string(), 1000.0, None),
            ("payment".to_string(), 300.0, None),
            ("credit_memo".to_string(), 100.0, None),
            ("finance_charge".to_string(), 50.0, None),
        ];

        compute_running_balances(&mut lines, 500.0);

        assert!((lines[0].2.unwrap() - 1500.0).abs() < 0.01); // 500 + 1000
        assert!((lines[1].2.unwrap() - 1200.0).abs() < 0.01); // 1500 - 300
        assert!((lines[2].2.unwrap() - 1100.0).abs() < 0.01); // 1200 - 100
        assert!((lines[3].2.unwrap() - 1150.0).abs() < 0.01); // 1100 + 50
    }

    #[test]
    fn test_running_balances_with_adjustments() {
        let mut lines: Vec<(String, f64, Option<f64>)> = vec![
            ("invoice".to_string(), 500.0, None),
            ("adjustment".to_string(), -50.0, None),
        ];

        compute_running_balances(&mut lines, 0.0);

        assert!((lines[0].2.unwrap() - 500.0).abs() < 0.01);
        assert!((lines[1].2.unwrap() - 450.0).abs() < 0.01); // 500 + (-50)
    }

    #[test]
    fn test_running_balances_opening_closing_unchanged() {
        let mut lines: Vec<(String, f64, Option<f64>)> = vec![
            ("opening_balance".to_string(), 1000.0, None),
            ("closing_balance".to_string(), 750.0, None),
        ];

        compute_running_balances(&mut lines, 750.0);

        assert!((lines[0].2.unwrap() - 750.0).abs() < 0.01); // unchanged
        assert!((lines[1].2.unwrap() - 750.0).abs() < 0.01); // unchanged
    }

    #[test]
    fn test_running_balances_empty() {
        let mut lines: Vec<(String, f64, Option<f64>)> = vec![];
        compute_running_balances(&mut lines, 1000.0);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_aging_breakdown_all_current() {
        let ref_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let items = vec![
            (chrono::NaiveDate::from_ymd_opt(2024, 7, 15).unwrap(), 500.0), // not yet due
            (chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(), 300.0), // due today
        ];
        let (cur, a30, a60, a90, a120, a121) = compute_aging_breakdown(ref_date, &items);
        assert!((cur - 800.0).abs() < 0.01);
        assert!((a30).abs() < 0.01);
        assert!((a60).abs() < 0.01);
        assert!((a90).abs() < 0.01);
        assert!((a120).abs() < 0.01);
        assert!((a121).abs() < 0.01);
    }

    #[test]
    fn test_aging_breakdown_distributed() {
        let ref_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let items = vec![
            (chrono::NaiveDate::from_ymd_opt(2024, 7, 15).unwrap(), 100.0), // current (not due)
            (chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), 200.0), // 15 days overdue -> 1-30
            (chrono::NaiveDate::from_ymd_opt(2024, 5, 15).unwrap(), 300.0), // 46 days -> 31-60
            (chrono::NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(), 400.0), // 76 days -> 61-90
            (chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), 500.0), // 107 days -> 91-120
            (chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 600.0),  // 181 days -> 121+
        ];
        let (cur, a30, a60, a90, a120, a121) = compute_aging_breakdown(ref_date, &items);
        assert!((cur - 100.0).abs() < 0.01);
        assert!((a30 - 200.0).abs() < 0.01);
        assert!((a60 - 300.0).abs() < 0.01);
        assert!((a90 - 400.0).abs() < 0.01);
        assert!((a120 - 500.0).abs() < 0.01);
        assert!((a121 - 600.0).abs() < 0.01);
    }

    #[test]
    fn test_aging_breakdown_skips_zero_and_negative() {
        let ref_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let items = vec![
            (chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(), 0.0),     // skip zero
            (chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(), -100.0),  // skip negative
            (chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(), 500.0),   // include
        ];
        let (cur, a30, a60, a90, a120, a121) = compute_aging_breakdown(ref_date, &items);
        assert!((a30 - 500.0).abs() < 0.01); // 29 days overdue -> 1-30
        assert!((cur).abs() < 0.01);
    }

    #[test]
    fn test_aging_breakdown_empty() {
        let ref_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let (cur, a30, a60, a90, a120, a121) = compute_aging_breakdown(ref_date, &[]);
        assert!((cur).abs() < 0.01);
        assert!((a30).abs() < 0.01);
        assert!((a60).abs() < 0.01);
        assert!((a90).abs() < 0.01);
        assert!((a120).abs() < 0.01);
        assert!((a121).abs() < 0.01);
    }

    #[test]
    fn test_closing_balance_full_scenario() {
        // Realistic scenario: customer had 5000 opening balance,
        // charged 2500 in new invoices, paid 3000, got 200 credit, 100 adjustment
        // closing = 5000 + 2500 - 3000 - 200 + 100 = 4400
        let closing = calculate_closing_balance(5000.0, 2500.0, 3000.0, 200.0, 100.0);
        assert!((closing - 4400.0).abs() < 0.01);

        let due = calculate_amount_due(closing);
        assert!((due - 4400.0).abs() < 0.01);
    }
}
