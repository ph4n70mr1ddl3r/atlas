//! Third-Party Payment Engine
//!
//! Manages the full lifecycle of third-party payments in Accounts Payable:
//! - Create payment instructions with type (garnishment, tax_levy, insurance, etc.)
//! - Approval workflow: draft → submitted → approved → paid
//! - Hold/release mechanism with audit trail
//! - Payment lines for GL account breakdown
//! - Dashboard and reporting
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Third-Party Payments

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_PAYMENT_TYPES: &[&str] = &["garnishment", "tax_levy", "insurance", "court_order", "custom"];
const VALID_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "paid", "on_hold", "cancelled", "rejected",
];
const VALID_LINE_TYPES: &[&str] = &["principal", "interest", "fee", "penalty", "adjustment"];

pub struct ThirdPartyPaymentEngine {
    repository: Arc<dyn ThirdPartyPaymentRepository>,
}

impl ThirdPartyPaymentEngine {
    pub fn new(repository: Arc<dyn ThirdPartyPaymentRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // CRUD
    // ========================================================================

    /// Create a new third-party payment instruction
    #[allow(clippy::too_many_arguments)]
    pub async fn create_payment(
        &self,
        org_id: Uuid,
        payment_number: &str,
        payment_type: &str,
        source_entity_type: &str,
        source_entity_id: Uuid,
        source_entity_name: &str,
        payee_name: &str,
        payee_tax_id: Option<&str>,
        payee_address: Option<&str>,
        payee_bank_account: Option<&str>,
        amount: &str,
        currency_code: &str,
        payment_method: Option<&str>,
        payment_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        reference_document: Option<&str>,
        reference_document_id: Option<Uuid>,
        description: Option<&str>,
        case_number: Option<&str>,
        court_jurisdiction: Option<&str>,
        is_recurring: bool,
        recurrence_frequency: Option<&str>,
        recurrence_start_date: Option<chrono::NaiveDate>,
        recurrence_end_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ThirdPartyPayment> {
        // Validate required fields
        if payment_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Payment number is required".into()));
        }
        if payee_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Payee name is required".into()));
        }
        if source_entity_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Source entity name is required".into()));
        }
        if !VALID_PAYMENT_TYPES.contains(&payment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment type '{}'. Must be one of: {}", payment_type, VALID_PAYMENT_TYPES.join(", ")
            )));
        }

        let amt: f64 = amount.parse().unwrap_or(-1.0);
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Amount must be positive".into()));
        }

        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        // Recurring payments require frequency and date range
        if is_recurring {
            if recurrence_frequency.is_none() || recurrence_frequency.unwrap().is_empty() {
                return Err(AtlasError::ValidationFailed(
                    "Recurring payments require a recurrence frequency".into(),
                ));
            }
            if recurrence_start_date.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "Recurring payments require a start date".into(),
                ));
            }
            if recurrence_end_date.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "Recurring payments require an end date".into(),
                ));
            }
        }

        // Check for duplicate payment number
        if self.repository.get_payment_by_number(org_id, payment_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Payment number '{}' already exists", payment_number
            )));
        }

        info!(
            "Creating third-party payment {} (type: {}) for payee {} on behalf of {}",
            payment_number, payment_type, payee_name, source_entity_name
        );

        self.repository.create_payment(
            org_id, payment_number, payment_type,
            source_entity_type, source_entity_id, source_entity_name,
            payee_name, payee_tax_id, payee_address, payee_bank_account,
            amount, currency_code, payment_method, payment_date, due_date,
            reference_document, reference_document_id, description,
            case_number, court_jurisdiction, is_recurring,
            recurrence_frequency, recurrence_start_date, recurrence_end_date,
            created_by,
        ).await
    }

    /// Get payment by ID
    pub async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<ThirdPartyPayment>> {
        self.repository.get_payment(id).await
    }

    /// Get payment by number
    pub async fn get_payment_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ThirdPartyPayment>> {
        self.repository.get_payment_by_number(org_id, number).await
    }

    /// List payments with optional filters
    pub async fn list_payments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        payment_type: Option<&str>,
        source_entity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ThirdPartyPayment>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = payment_type {
            if !VALID_PAYMENT_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid payment type '{}'. Must be one of: {}", t, VALID_PAYMENT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_payments(org_id, status, payment_type, source_entity_id).await
    }

    // ========================================================================
    // Workflow
    // ========================================================================

    /// Submit payment for approval
    pub async fn submit_payment(&self, payment_id: Uuid) -> AtlasResult<ThirdPartyPayment> {
        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit payment in '{}' status. Must be 'draft'.", p.status)
            ));
        }

        info!("Submitting third-party payment {} for approval", p.payment_number);
        self.repository.submit_payment(payment_id).await
    }

    /// Approve a submitted payment
    pub async fn approve_payment(&self, payment_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve payment in '{}' status. Must be 'submitted'.", p.status)
            ));
        }

        info!("Approving third-party payment {}", p.payment_number);
        self.repository.approve_payment(payment_id, approved_by).await
    }

    /// Reject a submitted payment
    pub async fn reject_payment(&self, payment_id: Uuid, reason: &str, rejected_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Rejection reason is required".into()));
        }

        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject payment in '{}' status. Must be 'submitted'.", p.status)
            ));
        }

        info!("Rejecting third-party payment {} (reason: {})", p.payment_number, reason);
        self.repository.reject_payment(payment_id, Some(reason), rejected_by).await
    }

    /// Place a payment on hold
    pub async fn place_on_hold(&self, payment_id: Uuid, reason: Option<&str>, held_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status == "cancelled" || p.status == "paid" || p.status == "rejected" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot hold payment in '{}' status.", p.status)
            ));
        }

        info!("Placing third-party payment {} on hold", p.payment_number);
        self.repository.place_on_hold(payment_id, reason, held_by).await
    }

    /// Release a payment from hold (returns to previous working state)
    pub async fn release_hold(&self, payment_id: Uuid) -> AtlasResult<ThirdPartyPayment> {
        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status != "on_hold" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot release payment in '{}' status. Must be 'on_hold'.", p.status)
            ));
        }

        info!("Releasing hold on third-party payment {}", p.payment_number);
        self.repository.release_hold(payment_id).await
    }

    /// Record a payment as paid
    pub async fn record_payment(&self, payment_id: Uuid, payment_ref: &str, paid_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
        if payment_ref.is_empty() {
            return Err(AtlasError::ValidationFailed("Payment reference is required".into()));
        }

        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot record payment in '{}' status. Must be 'approved'.", p.status)
            ));
        }

        info!("Recording payment for third-party payment {} (ref: {})", p.payment_number, payment_ref);
        self.repository.record_payment(payment_id, payment_ref, paid_by).await
    }

    /// Cancel a payment
    pub async fn cancel_payment(&self, payment_id: Uuid, reason: Option<&str>, cancelled_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status == "paid" {
            return Err(AtlasError::WorkflowError(
                "Cannot cancel a payment that has already been paid".into()
            ));
        }

        info!("Cancelling third-party payment {}", p.payment_number);
        self.repository.cancel_payment(payment_id, reason, cancelled_by).await
    }

    // ========================================================================
    // Lines
    // ========================================================================

    /// Add a payment line
    pub async fn add_line(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        line_number: i32,
        line_type: &str,
        description: Option<&str>,
        amount: &str,
        gl_account: Option<&str>,
        cost_center: Option<&str>,
        tax_code: Option<&str>,
    ) -> AtlasResult<ThirdPartyPaymentLine> {
        let p = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", payment_id)))?;

        if p.status == "cancelled" || p.status == "paid" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to payment in '{}' status.", p.status)
            ));
        }

        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line type '{}'. Must be one of: {}", line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        let line_amt: f64 = amount.parse().unwrap_or(-1.0);
        if line_amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Line amount must be positive".into()));
        }

        if line_number < 1 {
            return Err(AtlasError::ValidationFailed("Line number must be positive".into()));
        }

        info!("Adding line {} to third-party payment {}", line_number, p.payment_number);
        self.repository.add_line(
            org_id, payment_id, line_number, line_type, description,
            amount, gl_account, cost_center, tax_code,
        ).await
    }

    /// List payment lines
    pub async fn list_lines(&self, payment_id: Uuid) -> AtlasResult<Vec<ThirdPartyPaymentLine>> {
        self.repository.list_lines(payment_id).await
    }

    /// Remove a payment line
    pub async fn remove_line(&self, line_id: Uuid) -> AtlasResult<()> {
        self.repository.remove_line(line_id).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ThirdPartyPaymentDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        payments: std::sync::Mutex<Vec<ThirdPartyPayment>>,
        lines: std::sync::Mutex<Vec<ThirdPartyPaymentLine>>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                payments: std::sync::Mutex::new(vec![]),
                lines: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl ThirdPartyPaymentRepository for MockRepo {
        async fn create_payment(
            &self, org_id: Uuid, pn: &str, pt: &str, set: &str, sei: Uuid, sen: &str,
            payn: &str, pti: Option<&str>, pa: Option<&str>, pba: Option<&str>,
            amt: &str, cc: &str, pm: Option<&str>, pd: Option<chrono::NaiveDate>,
            dd: Option<chrono::NaiveDate>, rd: Option<&str>, rdi: Option<Uuid>,
            desc: Option<&str>, cn: Option<&str>, cj: Option<&str>,
            ir: bool, rf: Option<&str>, rsd: Option<chrono::NaiveDate>,
            red: Option<chrono::NaiveDate>, cb: Option<Uuid>,
        ) -> AtlasResult<ThirdPartyPayment> {
            let p = ThirdPartyPayment {
                id: Uuid::new_v4(),
                organization_id: org_id,
                payment_number: pn.into(),
                payment_type: pt.into(),
                status: "draft".into(),
                source_entity_type: set.into(),
                source_entity_id: sei,
                source_entity_name: sen.into(),
                payee_name: payn.into(),
                payee_tax_id: pti.map(Into::into),
                payee_address: pa.map(Into::into),
                payee_bank_account: pba.map(Into::into),
                amount: amt.into(),
                currency_code: cc.into(),
                payment_method: pm.map(Into::into),
                payment_date: pd,
                due_date: dd,
                reference_document: rd.map(Into::into),
                reference_document_id: rdi,
                description: desc.map(Into::into),
                case_number: cn.map(Into::into),
                court_jurisdiction: cj.map(Into::into),
                is_recurring: ir,
                recurrence_frequency: rf.map(Into::into),
                recurrence_start_date: rsd,
                recurrence_end_date: red,
                approved_by: None, approved_at: None,
                hold_reason: None, hold_at: None, hold_by: None,
                payment_reference: None, paid_at: None, paid_by: None,
                cancellation_reason: None, cancelled_at: None, cancelled_by: None,
                rejection_reason: None, rejected_by: None, rejected_at: None,
                metadata: serde_json::json!({}),
                created_by: cb,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            self.payments.lock().unwrap().push(p.clone());
            Ok(p)
        }

        async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<ThirdPartyPayment>> {
            Ok(self.payments.lock().unwrap().iter().find(|p| p.id == id).cloned())
        }

        async fn get_payment_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<ThirdPartyPayment>> {
            Ok(self.payments.lock().unwrap().iter()
                .find(|p| p.organization_id == org_id && p.payment_number == num).cloned())
        }

        async fn list_payments(&self, org_id: Uuid, status: Option<&str>, payment_type: Option<&str>, source_entity_id: Option<Uuid>) -> AtlasResult<Vec<ThirdPartyPayment>> {
            Ok(self.payments.lock().unwrap().iter()
                .filter(|p| p.organization_id == org_id)
                .filter(|p| status.is_none_or(|s| p.status == s))
                .filter(|p| payment_type.is_none_or(|t| p.payment_type == t))
                .filter(|p| source_entity_id.is_none_or(|id| p.source_entity_id == id))
                .cloned().collect())
        }

        async fn update_status(&self, id: Uuid, status: &str) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = status.into();
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn submit_payment(&self, id: Uuid) -> AtlasResult<ThirdPartyPayment> {
            self.update_status(id, "submitted").await
        }

        async fn approve_payment(&self, id: Uuid, ab: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = "approved".into();
            p.approved_by = ab;
            p.approved_at = Some(chrono::Utc::now());
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn reject_payment(&self, id: Uuid, reason: Option<&str>, rb: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = "rejected".into();
            p.rejection_reason = reason.map(Into::into);
            p.rejected_by = rb;
            p.rejected_at = Some(chrono::Utc::now());
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn place_on_hold(&self, id: Uuid, reason: Option<&str>, hb: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = "on_hold".into();
            p.hold_reason = reason.map(Into::into);
            p.hold_by = hb;
            p.hold_at = Some(chrono::Utc::now());
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn release_hold(&self, id: Uuid) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = "submitted".into();
            p.hold_reason = None;
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn record_payment(&self, id: Uuid, pref: &str, pb: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = "paid".into();
            p.payment_reference = Some(pref.into());
            p.paid_by = pb;
            p.paid_at = Some(chrono::Utc::now());
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn cancel_payment(&self, id: Uuid, reason: Option<&str>, cb: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> {
            let mut ps = self.payments.lock().unwrap();
            let p = ps.iter_mut().find(|p| p.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Payment {} not found", id)))?;
            p.status = "cancelled".into();
            p.cancellation_reason = reason.map(Into::into);
            p.cancelled_by = cb;
            p.cancelled_at = Some(chrono::Utc::now());
            p.updated_at = chrono::Utc::now();
            Ok(p.clone())
        }

        async fn add_line(&self, org_id: Uuid, pid: Uuid, ln: i32, lt: &str, desc: Option<&str>, amt: &str, gla: Option<&str>, cc: Option<&str>, tc: Option<&str>) -> AtlasResult<ThirdPartyPaymentLine> {
            let line = ThirdPartyPaymentLine {
                id: Uuid::new_v4(),
                organization_id: org_id,
                payment_id: pid,
                line_number: ln,
                line_type: lt.into(),
                description: desc.map(Into::into),
                amount: amt.into(),
                gl_account: gla.map(Into::into),
                cost_center: cc.map(Into::into),
                tax_code: tc.map(Into::into),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            self.lines.lock().unwrap().push(line.clone());
            Ok(line)
        }

        async fn list_lines(&self, payment_id: Uuid) -> AtlasResult<Vec<ThirdPartyPaymentLine>> {
            Ok(self.lines.lock().unwrap().iter().filter(|l| l.payment_id == payment_id).cloned().collect())
        }

        async fn get_line(&self, line_id: Uuid) -> AtlasResult<Option<ThirdPartyPaymentLine>> {
            Ok(self.lines.lock().unwrap().iter().find(|l| l.id == line_id).cloned())
        }

        async fn remove_line(&self, line_id: Uuid) -> AtlasResult<()> {
            let mut ls = self.lines.lock().unwrap();
            ls.retain(|l| l.id != line_id);
            Ok(())
        }

        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<ThirdPartyPaymentDashboard> {
            Ok(ThirdPartyPaymentDashboard {
                total_payments: 0, draft_count: 0, pending_approval_count: 0,
                approved_count: 0, paid_count: 0, on_hold_count: 0, cancelled_count: 0,
                total_amount: "0.00".into(), paid_amount: "0.00".into(),
                pending_amount: "0.00".into(), on_hold_amount: "0.00".into(),
                payments_by_type: serde_json::json!([]), upcoming_due: serde_json::json!([]),
            })
        }
    }

    fn eng() -> ThirdPartyPaymentEngine {
        ThirdPartyPaymentEngine::new(Arc::new(MockRepo::new()))
    }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_PAYMENT_TYPES.len(), 5);
        assert_eq!(VALID_STATUSES.len(), 7);
        assert_eq!(VALID_LINE_TYPES.len(), 5);
    }

    #[tokio::test]
    async fn test_create_garnishment_payment() {
        let p = eng().create_payment(
            Uuid::new_v4(), "TPP-001", "garnishment", "supplier",
            Uuid::new_v4(), "John Smith", "IRS", Some("12-3456789"),
            None, None, "5000.00", "USD", Some("wire"), None, None,
            None, None, Some("Wage garnishment for back taxes"),
            Some("Case-2024-001"), Some("Federal"),
            false, None, None, None, None,
        ).await.unwrap();
        assert_eq!(p.payment_number, "TPP-001");
        assert_eq!(p.payment_type, "garnishment");
        assert_eq!(p.status, "draft");
        assert_eq!(p.payee_name, "IRS");
    }

    #[tokio::test]
    async fn test_create_tax_levy_payment() {
        let p = eng().create_payment(
            Uuid::new_v4(), "TPP-002", "tax_levy", "employee",
            Uuid::new_v4(), "Jane Doe", "State Tax Board", None,
            None, None, "2500.00", "USD", None, None, None,
            None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        assert_eq!(p.payment_type, "tax_levy");
    }

    #[tokio::test]
    async fn test_create_recurring_payment() {
        let p = eng().create_payment(
            Uuid::new_v4(), "TPP-REC-001", "insurance", "supplier",
            Uuid::new_v4(), "Acme Corp", "Global Insurance Co", None,
            None, None, "1200.00", "USD", None, None, None,
            None, None, Some("Monthly liability insurance"),
            None, None,
            true, Some("monthly"), Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()), None,
        ).await.unwrap();
        assert!(p.is_recurring);
        assert_eq!(p.recurrence_frequency, Some("monthly".into()));
    }

    #[tokio::test]
    async fn test_create_recurring_missing_frequency_fails() {
        let r = eng().create_payment(
            Uuid::new_v4(), "TPP-REC-ERR", "insurance", "supplier",
            Uuid::new_v4(), "Acme Corp", "Insurance Co", None,
            None, None, "1200.00", "USD", None, None, None,
            None, None, None, None, None,
            true, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_payment_empty_number_fails() {
        let r = eng().create_payment(
            Uuid::new_v4(), "", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None,
            None, None, "100.00", "USD", None, None, None,
            None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_payment_empty_payee_fails() {
        let r = eng().create_payment(
            Uuid::new_v4(), "TPP-X", "garnishment", "supplier",
            Uuid::new_v4(), "John", "", None,
            None, None, "100.00", "USD", None, None, None,
            None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_payment_invalid_type_fails() {
        let r = eng().create_payment(
            Uuid::new_v4(), "TPP-X", "bribe", "supplier",
            Uuid::new_v4(), "John", "Acme", None,
            None, None, "100.00", "USD", None, None, None,
            None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_payment_zero_amount_fails() {
        let r = eng().create_payment(
            Uuid::new_v4(), "TPP-X", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None,
            None, None, "0.00", "USD", None, None, None,
            None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_payment_invalid_currency_fails() {
        let r = eng().create_payment(
            Uuid::new_v4(), "TPP-X", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None,
            None, None, "100.00", "US", None, None, None,
            None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_payment_duplicate_number_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_payment(
            org, "TPP-DUP", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        let r = e.create_payment(
            org, "TPP-DUP", "tax_levy", "employee",
            Uuid::new_v4(), "Jane", "State", None, None, None,
            "200.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    // ========================================================================
    // Workflow Tests
    // ========================================================================

    #[tokio::test]
    async fn test_full_workflow_draft_to_paid() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-WF-001", "garnishment", "supplier",
            Uuid::new_v4(), "John Smith", "IRS", None, None, None,
            "5000.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        assert_eq!(p.status, "draft");

        let p = e.submit_payment(p.id).await.unwrap();
        assert_eq!(p.status, "submitted");

        let p = e.approve_payment(p.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(p.status, "approved");
        assert!(p.approved_by.is_some());

        let p = e.record_payment(p.id, "PAY-REF-001", Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(p.status, "paid");
        assert_eq!(p.payment_reference, Some("PAY-REF-001".into()));
    }

    #[tokio::test]
    async fn test_submit_non_draft_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-SUB-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.submit_payment(p.id).await.unwrap(); // draft → submitted
        let r = e.submit_payment(p.id).await; // submitted → can't submit again
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_approve_non_submitted_fails() {
        let e = eng();
        let p = eng().create_payment(
            Uuid::new_v4(), "TPP-APR-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let r = e.approve_payment(p.id, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_reject_submitted() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-REJ-001", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.submit_payment(p.id).await.unwrap();
        let p = e.reject_payment(p.id, "Insufficient documentation", None).await.unwrap();
        assert_eq!(p.status, "rejected");
        assert_eq!(p.rejection_reason, Some("Insufficient documentation".into()));
    }

    #[tokio::test]
    async fn test_reject_without_reason_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-REJ-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.submit_payment(p.id).await.unwrap();
        let r = e.reject_payment(p.id, "", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_hold_and_release() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-HOLD-001", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let p = e.place_on_hold(p.id, Some("Pending court order review"), None).await.unwrap();
        assert_eq!(p.status, "on_hold");
        assert_eq!(p.hold_reason, Some("Pending court order review".into()));

        let p = e.release_hold(p.id).await.unwrap();
        assert_eq!(p.status, "submitted");
        assert!(p.hold_reason.is_none());
    }

    #[tokio::test]
    async fn test_hold_paid_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-HOLD-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.submit_payment(p.id).await.unwrap();
        e.approve_payment(p.id, None).await.unwrap();
        e.record_payment(p.id, "REF-001", None).await.unwrap();
        let r = e.place_on_hold(p.id, Some("test"), None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_draft() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-CAN-001", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let p = e.cancel_payment(p.id, Some("No longer required"), None).await.unwrap();
        assert_eq!(p.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_paid_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-CAN-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.submit_payment(p.id).await.unwrap();
        e.approve_payment(p.id, None).await.unwrap();
        e.record_payment(p.id, "REF-001", None).await.unwrap();
        let r = e.cancel_payment(p.id, None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_record_payment_empty_ref_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-REC-REF", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.submit_payment(p.id).await.unwrap();
        e.approve_payment(p.id, None).await.unwrap();
        let r = e.record_payment(p.id, "", None).await;
        assert!(r.is_err());
    }

    // ========================================================================
    // Line Tests
    // ========================================================================

    #[tokio::test]
    async fn test_add_line() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-LINE-001", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "5000.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let line = e.add_line(
            p.organization_id, p.id, 1, "principal",
            Some("Principal garnishment amount"), "4000.00",
            Some("2100-100"), Some("LEGAL"), None,
        ).await.unwrap();
        assert_eq!(line.line_type, "principal");
        assert_eq!(line.amount, "4000.00");
    }

    #[tokio::test]
    async fn test_add_line_invalid_type_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-LINE-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let r = e.add_line(
            p.organization_id, p.id, 1, "bogus",
            None, "100.00", None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_add_line_zero_amount_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-LINE-ZERO", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let r = e.add_line(
            p.organization_id, p.id, 1, "principal",
            None, "0.00", None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_add_line_to_cancelled_payment_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-LINE-CAN", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        e.cancel_payment(p.id, Some("Cancelled"), None).await.unwrap();
        let r = e.add_line(
            p.organization_id, p.id, 1, "principal",
            None, "50.00", None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_payments_filter() {
        let r = eng().list_payments(Uuid::new_v4(), Some("draft"), Some("garnishment"), None).await;
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_payments_invalid_status() {
        let r = eng().list_payments(Uuid::new_v4(), Some("unknown"), None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_payments, 0);
        assert_eq!(d.total_amount, "0.00");
    }

    #[tokio::test]
    async fn test_get_payment_not_found() {
        let r = eng().get_payment(Uuid::new_v4()).await.unwrap();
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn test_release_hold_non_held_fails() {
        let e = eng();
        let p = e.create_payment(
            Uuid::new_v4(), "TPP-REL-ERR", "garnishment", "supplier",
            Uuid::new_v4(), "John", "IRS", None, None, None,
            "100.00", "USD", None, None, None, None, None, None, None, None,
            false, None, None, None, None,
        ).await.unwrap();
        let r = e.release_hold(p.id).await;
        assert!(r.is_err());
    }
}
