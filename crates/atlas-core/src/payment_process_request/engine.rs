//! Payment Process Request Engine
//!
//! Orchestrates PPR creation, lifecycle transitions, document management,
//! activity tracking, validation, and dashboard summary.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Payment Process Requests

use super::repository::{PaymentProcessRequest, PaymentProcessRequestRepository};
use atlas_shared::{AtlasError, AtlasResult};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid statuses
const VALID_STATUSES: &[&str] = &[
    "draft",
    "submitted",
    "selection_complete",
    "formatted",
    "confirmed",
    "cancelled",
];

const VALID_PAYMENT_METHODS: &[&str] = &["check", "electronic", "wire", "ach", "swift", "all"];

const VALID_SELECTION_CRITERIA: &[&str] = &[
    "due_date",
    "discount_date",
    "all_open",
    "supplier",
    "pay_group",
];

/// Payment Process Request Engine
pub struct PaymentProcessRequestEngine {
    repo: Arc<dyn PaymentProcessRequestRepository>,
}

impl PaymentProcessRequestEngine {
    pub fn new(repo: Arc<dyn PaymentProcessRequestRepository>) -> Self {
        Self { repo }
    }

    fn validate_payment_method(method: &str) -> AtlasResult<()> {
        if !VALID_PAYMENT_METHODS.contains(&method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment_method '{}'. Must be one of: {}",
                method,
                VALID_PAYMENT_METHODS.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_selection_criteria(criteria: &str) -> AtlasResult<()> {
        if !VALID_SELECTION_CRITERIA.contains(&criteria) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid selection_criteria '{}'. Must be one of: {}",
                criteria,
                VALID_SELECTION_CRITERIA.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_status(status: &str) -> AtlasResult<()> {
        if !VALID_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}",
                status,
                VALID_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    /// Validate lifecycle status transition
    /// draft → submitted → `selection_complete` → formatted → confirmed
    /// draft / submitted / `selection_complete` / formatted → cancelled
    pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("draft", "submitted") => Ok(()),
            ("draft", "cancelled") => Ok(()),
            ("submitted", "selection_complete") => Ok(()),
            ("submitted", "cancelled") => Ok(()),
            ("selection_complete", "formatted") => Ok(()),
            ("selection_complete", "cancelled") => Ok(()),
            ("formatted", "confirmed") => Ok(()),
            ("formatted", "cancelled") => Ok(()),
            _ => Err(AtlasError::ValidationFailed(format!(
                "Invalid status transition from '{current}' to '{target}'"
            ))),
        }
    }

    // ========================================================================
    // CRUD
    // ========================================================================

    pub async fn create_request(
        &self,
        org_id: Uuid,
        request_name: &str,
        description: Option<&str>,
        payment_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        payment_method: &str,
        currency_code: &str,
        exchange_rate_type: Option<&str>,
        exchange_rate: Option<f64>,
        selection_criteria: &str,
        due_date_from: Option<chrono::NaiveDate>,
        due_date_to: Option<chrono::NaiveDate>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        pay_group: Option<&str>,
        minimum_amount: Option<f64>,
        maximum_amount: Option<f64>,
        include_on_hold: bool,
        take_discount: bool,
        pay_only_due: bool,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_document: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentProcessRequest> {
        Self::validate_payment_method(payment_method)?;
        Self::validate_selection_criteria(selection_criteria)?;

        // Validate date range when using due_date criteria
        if selection_criteria == "due_date" {
            if due_date_from.is_none() || due_date_to.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "Due date range is required for due_date selection criteria".to_string(),
                ));
            }
            if let (Some(from), Some(to)) = (due_date_from, due_date_to) {
                if from > to {
                    return Err(AtlasError::ValidationFailed(
                        "Due date from must be before due date to".to_string(),
                    ));
                }
            }
        }

        if gl_date > payment_date {
            return Err(AtlasError::ValidationFailed(
                "GL date cannot be after payment date".to_string(),
            ));
        }

        // Generate request number
        let request_number = format!("PPR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%.f"));

        let ppr = self
            .repo
            .create_request(
                org_id,
                &request_number,
                request_name,
                description,
                payment_date,
                gl_date,
                payment_method,
                currency_code,
                exchange_rate_type,
                exchange_rate,
                selection_criteria,
                due_date_from,
                due_date_to,
                supplier_id,
                supplier_name,
                pay_group,
                minimum_amount,
                maximum_amount,
                include_on_hold,
                take_discount,
                pay_only_due,
                bank_account_id,
                bank_account_name,
                payment_document,
                created_by,
            )
            .await?;

        // Log creation activity
        self.repo
            .log_activity(
                org_id,
                ppr.id,
                None,
                "created",
                Some(&format!("Created PPR {request_number}")),
                None,
                Some("draft"),
                created_by,
                None,
                serde_json::json!({"requestName": request_name}),
            )
            .await
            .ok();

        info!("PPR: Created request {} ({})", request_number, request_name);
        Ok(ppr)
    }

    pub async fn get_request(&self, org_id: Uuid, id: Uuid) -> AtlasResult<PaymentProcessRequest> {
        self.repo.get_request(org_id, id).await
    }

    pub async fn get_request_by_number(
        &self,
        org_id: Uuid,
        request_number: &str,
    ) -> AtlasResult<PaymentProcessRequest> {
        self.repo
            .get_request_by_number(org_id, request_number)
            .await
    }

    pub async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PaymentProcessRequest>> {
        if let Some(s) = status {
            Self::validate_status(s)?;
        }
        self.repo.list_requests(org_id, status).await
    }

    pub async fn delete_request(&self, org_id: Uuid, request_number: &str) -> AtlasResult<()> {
        self.repo.delete_request(org_id, request_number).await
    }

    // ========================================================================
    // Lifecycle
    // ========================================================================

    pub async fn submit_request(
        &self,
        org_id: Uuid,
        id: Uuid,
        submitted_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repo.get_request(org_id, id).await?;
        Self::validate_status_transition(&ppr.status, "submitted")?;

        if ppr.total_documents == 0 {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit PPR without selected documents".to_string(),
            ));
        }

        let updated = self.repo.set_submitted(id, submitted_by).await?;

        self.repo
            .log_activity(
                org_id,
                id,
                None,
                "submitted",
                Some("PPR submitted for processing"),
                Some("draft"),
                Some("submitted"),
                Some(submitted_by),
                None,
                serde_json::json!({}),
            )
            .await
            .ok();

        info!("PPR: Submitted request {}", ppr.request_number);
        Ok(updated)
    }

    pub async fn complete_selection(
        &self,
        org_id: Uuid,
        id: Uuid,
        completed_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repo.get_request(org_id, id).await?;
        Self::validate_status_transition(&ppr.status, "selection_complete")?;

        let updated = self.repo.set_selection_complete(id, completed_by).await?;

        self.repo
            .log_activity(
                org_id,
                id,
                None,
                "selection_complete",
                Some("Invoice selection completed"),
                Some("submitted"),
                Some("selection_complete"),
                Some(completed_by),
                None,
                serde_json::json!({"totalDocuments": ppr.total_documents}),
            )
            .await
            .ok();

        info!("PPR: Selection completed for {}", ppr.request_number);
        Ok(updated)
    }

    pub async fn format_payments(
        &self,
        org_id: Uuid,
        id: Uuid,
        formatted_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repo.get_request(org_id, id).await?;
        Self::validate_status_transition(&ppr.status, "formatted")?;

        let updated = self.repo.set_formatted(id, formatted_by).await?;

        self.repo
            .log_activity(
                org_id,
                id,
                None,
                "formatted",
                Some("Payment formatting completed"),
                Some("selection_complete"),
                Some("formatted"),
                Some(formatted_by),
                None,
                serde_json::json!({}),
            )
            .await
            .ok();

        info!("PPR: Payments formatted for {}", ppr.request_number);
        Ok(updated)
    }

    pub async fn confirm_payments(
        &self,
        org_id: Uuid,
        id: Uuid,
        confirmed_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repo.get_request(org_id, id).await?;
        Self::validate_status_transition(&ppr.status, "confirmed")?;

        let updated = self.repo.set_confirmed(id, confirmed_by).await?;

        self.repo
            .log_activity(
                org_id,
                id,
                None,
                "confirmed",
                Some("Payments confirmed and processed"),
                Some("formatted"),
                Some("confirmed"),
                Some(confirmed_by),
                None,
                serde_json::json!({"totalPaymentAmount": ppr.total_payment_amount}),
            )
            .await
            .ok();

        info!("PPR: Payments confirmed for {}", ppr.request_number);
        Ok(updated)
    }

    pub async fn cancel_request(
        &self,
        org_id: Uuid,
        id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repo.get_request(org_id, id).await?;
        Self::validate_status_transition(&ppr.status, "cancelled")?;

        let old_status = ppr.status.clone();
        let updated = self.repo.set_cancelled(id, cancelled_by, reason).await?;

        self.repo
            .log_activity(
                org_id,
                id,
                None,
                "cancelled",
                Some(&format!(
                    "PPR cancelled{}",
                    reason.map(|r| format!(": {r}")).unwrap_or_default()
                )),
                Some(&old_status),
                Some("cancelled"),
                Some(cancelled_by),
                None,
                serde_json::json!({"reason": reason}),
            )
            .await
            .ok();

        info!("PPR: Cancelled request {}", ppr.request_number);
        Ok(updated)
    }

    // ========================================================================
    // Document Management
    // ========================================================================

    pub async fn add_document(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_amount: f64,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        original_amount: f64,
        amount_due: f64,
        amount_to_pay: f64,
        discount_available: f64,
        discount_taken: f64,
        discount_date: Option<chrono::NaiveDate>,
        currency_code: Option<&str>,
        liability_account: Option<&str>,
        discount_account: Option<&str>,
        cash_account: Option<&str>,
    ) -> AtlasResult<super::repository::PprSelectedDocument> {
        let ppr = self.repo.get_request(org_id, ppr_id).await?;
        if ppr.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only add documents to a draft PPR".to_string(),
            ));
        }

        if amount_to_pay <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount to pay must be positive".to_string(),
            ));
        }

        if amount_to_pay > amount_due {
            return Err(AtlasError::ValidationFailed(
                "Amount to pay cannot exceed amount due".to_string(),
            ));
        }

        if discount_taken > discount_available {
            return Err(AtlasError::ValidationFailed(
                "Discount taken cannot exceed discount available".to_string(),
            ));
        }

        let doc = self
            .repo
            .add_document(
                org_id,
                ppr_id,
                invoice_id,
                invoice_number,
                invoice_date,
                invoice_amount,
                supplier_id,
                supplier_number,
                supplier_name,
                supplier_site,
                original_amount,
                amount_due,
                amount_to_pay,
                discount_available,
                discount_taken,
                discount_date,
                currency_code,
                liability_account,
                discount_account,
                cash_account,
            )
            .await?;

        self.repo
            .log_activity(
                org_id,
                ppr_id,
                Some(doc.id),
                "document_added",
                Some(&format!(
                    "Added document for invoice {}",
                    invoice_number.unwrap_or("N/A")
                )),
                None,
                None,
                None,
                None,
                serde_json::json!({"invoiceAmount": invoice_amount}),
            )
            .await
            .ok();

        Ok(doc)
    }

    pub async fn list_documents(
        &self,
        ppr_id: Uuid,
    ) -> AtlasResult<Vec<super::repository::PprSelectedDocument>> {
        self.repo.list_documents(ppr_id).await
    }

    pub async fn remove_document(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        document_id: Uuid,
    ) -> AtlasResult<()> {
        let ppr = self.repo.get_request(org_id, ppr_id).await?;
        if ppr.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only remove documents from a draft PPR".to_string(),
            ));
        }
        self.repo.remove_document(ppr_id, document_id).await
    }

    // ========================================================================
    // Activities & Dashboard
    // ========================================================================

    pub async fn list_activities(
        &self,
        ppr_id: Uuid,
    ) -> AtlasResult<Vec<super::repository::PprActivity>> {
        self.repo.list_activities(ppr_id).await
    }

    pub async fn get_dashboard(
        &self,
        org_id: Uuid,
    ) -> AtlasResult<super::repository::PprDashboard> {
        self.repo.get_dashboard(org_id).await
    }
}
