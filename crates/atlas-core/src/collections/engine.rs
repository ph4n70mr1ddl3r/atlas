//! Collections & Credit Management Engine
//!
//! Manages customer credit profiles, credit limits, risk classification,
//! collection strategies, collection cases, customer interactions,
//! promise-to-pay tracking, dunning campaigns, receivables aging analysis,
//! and write-off management.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Collections

use atlas_shared::{
    CustomerCreditProfile, CollectionCase, CustomerInteraction, PromiseToPay,
    WriteOffRequest, AgingSummary,
    AtlasError, AtlasResult,
};
use super::CollectionsRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid risk classifications
const VALID_RISK_CLASSIFICATIONS: &[&str] = &[
    "low", "medium", "high", "very_high", "defaulted",
];

/// Valid case types
const VALID_CASE_TYPES: &[&str] = &[
    "collection", "dispute", "bankruptcy", "skip_trace",
];

/// Valid case statuses
const VALID_CASE_STATUSES: &[&str] = &[
    "open", "in_progress", "resolved", "closed", "escalated", "written_off",
];

/// Valid case priorities
const VALID_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

/// Valid interaction types
const VALID_INTERACTION_TYPES: &[&str] = &[
    "phone_call", "email", "letter", "meeting", "note", "sms",
];

/// Valid interaction outcomes
const VALID_OUTCOMES: &[&str] = &[
    "contacted", "left_message", "no_answer", "promised_to_pay",
    "disputed", "refused", "agreed_payment_plan", "escalated", "no_action",
];

/// Valid promise types
const VALID_PROMISE_TYPES: &[&str] = &[
    "single_payment", "installment", "full_balance",
];

/// Valid promise statuses
const VALID_PROMISE_STATUSES: &[&str] = &[
    "pending", "partially_kept", "kept", "broken", "cancelled",
];

/// Valid dunning levels
const VALID_DUNNING_LEVELS: &[&str] = &[
    "reminder", "first_notice", "second_notice", "final_notice", "pre_legal", "legal",
];

/// Valid communication methods
const VALID_COMMUNICATION_METHODS: &[&str] = &[
    "email", "letter", "sms", "phone",
];

/// Valid write-off types
const VALID_WRITE_OFF_TYPES: &[&str] = &[
    "bad_debt", "small_balance", "dispute", "adjustment",
];

/// Valid write-off statuses
const VALID_WRITE_OFF_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "processed", "cancelled",
];

/// Collections & Credit Management engine
pub struct CollectionsEngine {
    repository: Arc<dyn CollectionsRepository>,
}

impl CollectionsEngine {
    pub fn new(repository: Arc<dyn CollectionsRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Credit Profile Management
    // ========================================================================

    /// Create or update a customer credit profile
    pub async fn create_credit_profile(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        credit_limit: &str,
        risk_classification: &str,
        credit_score: Option<i32>,
        external_credit_rating: Option<&str>,
        external_rating_agency: Option<&str>,
        external_rating_date: Option<chrono::NaiveDate>,
        payment_terms: &str,
        next_review_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CustomerCreditProfile> {
        let limit: f64 = credit_limit.parse().map_err(|_| AtlasError::ValidationFailed(
            "Credit limit must be a valid number".to_string(),
        ))?;
        if limit < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Credit limit must be non-negative".to_string(),
            ));
        }
        if !VALID_RISK_CLASSIFICATIONS.contains(&risk_classification) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk classification '{}'. Must be one of: {}",
                risk_classification, VALID_RISK_CLASSIFICATIONS.join(", ")
            )));
        }
        if let Some(score) = credit_score {
            if !(0..=1000).contains(&score) {
                return Err(AtlasError::ValidationFailed(
                    "Credit score must be between 0 and 1000".to_string(),
                ));
            }
        }

        info!("Creating/updating credit profile for customer {} in org {}", customer_id, org_id);

        self.repository.create_credit_profile(
            org_id, customer_id, customer_number, customer_name,
            credit_limit, risk_classification, credit_score,
            external_credit_rating, external_rating_agency, external_rating_date,
            payment_terms, next_review_date, created_by,
        ).await
    }

    /// Get a customer's credit profile
    pub async fn get_credit_profile(&self, org_id: Uuid, customer_id: Uuid) -> AtlasResult<Option<CustomerCreditProfile>> {
        self.repository.get_credit_profile(org_id, customer_id).await
    }

    /// List credit profiles with optional filters
    pub async fn list_credit_profiles(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        risk_classification: Option<&str>,
    ) -> AtlasResult<Vec<CustomerCreditProfile>> {
        self.repository.list_credit_profiles(org_id, status, risk_classification).await
    }

    /// Place a customer on credit hold
    pub async fn place_credit_hold(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        reason: &str,
        placed_by: Uuid,
    ) -> AtlasResult<CustomerCreditProfile> {
        let profile = self.repository.get_credit_profile(org_id, customer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile not found for customer {}", customer_id)
            ))?;

        if profile.credit_hold {
            return Err(AtlasError::ValidationFailed(
                "Customer is already on credit hold".to_string(),
            ));
        }

        info!("Placing customer {} on credit hold: {}", customer_id, reason);

        self.repository.update_credit_profile(
            profile.id,
            None, None, None, None, None, None, None, None, None,
            Some(true), Some(reason), Some(chrono::Utc::now()), Some(placed_by),
            None, None, None,
        ).await
    }

    /// Remove a customer from credit hold
    pub async fn remove_credit_hold(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
    ) -> AtlasResult<CustomerCreditProfile> {
        let profile = self.repository.get_credit_profile(org_id, customer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile not found for customer {}", customer_id)
            ))?;

        if !profile.credit_hold {
            return Err(AtlasError::ValidationFailed(
                "Customer is not on credit hold".to_string(),
            ));
        }

        info!("Removing credit hold for customer {}", customer_id);

        self.repository.update_credit_profile(
            profile.id,
            None, None, None, None, None, None, None, None, None,
            Some(false), None, None, None,
            Some(chrono::Utc::now().date_naive()), None, None,
        ).await
    }

    /// Check if a customer can be extended additional credit
    pub async fn check_credit_available(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        additional_amount: &str,
    ) -> AtlasResult<bool> {
        let profile = self.repository.get_credit_profile(org_id, customer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile not found for customer {}", customer_id)
            ))?;

        if profile.credit_hold {
            return Ok(false);
        }

        let additional: f64 = additional_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Additional amount must be a valid number".to_string(),
        ))?;

        let available: f64 = profile.credit_available.parse().unwrap_or(0.0);

        Ok(additional <= available)
    }

    // ========================================================================
    // Collection Cases
    // ========================================================================

    /// Create a new collection case
    pub async fn create_case(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        strategy_id: Option<Uuid>,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        case_type: &str,
        priority: &str,
        total_overdue_amount: &str,
        total_disputed_amount: &str,
        total_invoiced_amount: &str,
        overdue_invoice_count: i32,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        related_invoice_ids: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CollectionCase> {
        if !VALID_CASE_TYPES.contains(&case_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid case type '{}'. Must be one of: {}",
                case_type, VALID_CASE_TYPES.join(", ")
            )));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}",
                priority, VALID_PRIORITIES.join(", ")
            )));
        }

        let case_number = format!("CC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating collection case {} for customer {}", case_number, customer_id);

        self.repository.create_case(
            org_id, &case_number, customer_id, customer_number, customer_name,
            strategy_id, assigned_to, assigned_to_name,
            case_type, priority,
            total_overdue_amount, total_disputed_amount, total_invoiced_amount,
            overdue_invoice_count, oldest_overdue_date,
            related_invoice_ids, created_by,
        ).await
    }

    /// Get a collection case by ID
    pub async fn get_case(&self, id: Uuid) -> AtlasResult<Option<CollectionCase>> {
        self.repository.get_case(id).await
    }

    /// List collection cases with optional filters
    pub async fn list_cases(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
        assigned_to: Option<Uuid>,
    ) -> AtlasResult<Vec<CollectionCase>> {
        if let Some(s) = status {
            if !VALID_CASE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CASE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_cases(org_id, status, customer_id, assigned_to).await
    }

    /// Escalate a collection case
    pub async fn escalate_case(
        &self,
        case_id: Uuid,
        escalated_to: Uuid,
        escalated_to_name: Option<&str>,
        reason: Option<&str>,
    ) -> AtlasResult<CollectionCase> {
        let case = self.repository.get_case(case_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Collection case {} not found", case_id)
            ))?;

        if case.status != "open" && case.status != "in_progress" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot escalate case in '{}' status", case.status)
            ));
        }

        info!("Escalating collection case {} to {}", case.case_number, escalated_to);

        let today = chrono::Utc::now().date_naive();
        self.repository.update_case_status(
            case_id,
            "escalated",
            None,
            Some(escalated_to),
            escalated_to_name,
            Some(today),
            None,
            None,
            reason,
            None,
            None,
        ).await
    }

    /// Resolve a collection case
    pub async fn resolve_case(
        &self,
        case_id: Uuid,
        resolution_type: &str,
        resolution_notes: Option<&str>,
    ) -> AtlasResult<CollectionCase> {
        let case = self.repository.get_case(case_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Collection case {} not found", case_id)
            ))?;

        if case.status == "closed" || case.status == "resolved" {
            return Err(AtlasError::WorkflowError(
                format!("Case is already '{}'", case.status)
            ));
        }

        let today = chrono::Utc::now().date_naive();
        info!("Resolving collection case {} as: {}", case.case_number, resolution_type);

        self.repository.update_case_status(
            case_id,
            "resolved",
            None,
            None,
            None,
            Some(today),
            None,
            Some(resolution_type),
            resolution_notes,
            Some(today),
            None,
        ).await
    }

    // ========================================================================
    // Customer Interactions
    // ========================================================================

    /// Record a customer interaction
    pub async fn record_interaction(
        &self,
        org_id: Uuid,
        case_id: Option<Uuid>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        interaction_type: &str,
        direction: &str,
        contact_name: Option<&str>,
        contact_role: Option<&str>,
        contact_phone: Option<&str>,
        contact_email: Option<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        outcome: Option<&str>,
        follow_up_date: Option<chrono::NaiveDate>,
        follow_up_notes: Option<&str>,
        performed_by: Option<Uuid>,
        performed_by_name: Option<&str>,
        duration_minutes: Option<i32>,
    ) -> AtlasResult<CustomerInteraction> {
        if !VALID_INTERACTION_TYPES.contains(&interaction_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid interaction type '{}'. Must be one of: {}",
                interaction_type, VALID_INTERACTION_TYPES.join(", ")
            )));
        }
        if !["outbound", "inbound"].contains(&direction) {
            return Err(AtlasError::ValidationFailed(
                "Direction must be 'outbound' or 'inbound'".to_string(),
            ));
        }
        if let Some(o) = outcome {
            if !VALID_OUTCOMES.contains(&o) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid outcome '{}'. Must be one of: {}", o, VALID_OUTCOMES.join(", ")
                )));
            }
        }

        info!("Recording {} interaction for customer {}", interaction_type, customer_id);

        // If linked to a case, update the case's last_action_date
        if case_id.is_some() {
            let today = chrono::Utc::now().date_naive();
            let _ = self.repository.update_case_status(
                case_id.unwrap(),
                "in_progress",
                None, None, None,
                Some(today), follow_up_date,
                None, None, None, None,
            ).await;
        }

        self.repository.create_interaction(
            org_id, case_id, customer_id, customer_number, customer_name,
            interaction_type, direction,
            contact_name, contact_role, contact_phone, contact_email,
            subject, body, outcome, follow_up_date, follow_up_notes,
            performed_by, performed_by_name, duration_minutes,
        ).await
    }

    /// List interactions for a case or customer
    pub async fn list_interactions(
        &self,
        org_id: Uuid,
        case_id: Option<Uuid>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CustomerInteraction>> {
        self.repository.list_interactions(org_id, case_id, customer_id).await
    }

    // ========================================================================
    // Promise to Pay
    // ========================================================================

    /// Record a promise to pay
    pub async fn create_promise_to_pay(
        &self,
        org_id: Uuid,
        case_id: Option<Uuid>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        promise_type: &str,
        promised_amount: &str,
        promise_date: chrono::NaiveDate,
        installment_count: Option<i32>,
        installment_frequency: Option<&str>,
        related_invoice_ids: serde_json::Value,
        promised_by_name: Option<&str>,
        promised_by_role: Option<&str>,
        notes: Option<&str>,
        recorded_by: Option<Uuid>,
    ) -> AtlasResult<PromiseToPay> {
        if !VALID_PROMISE_TYPES.contains(&promise_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid promise type '{}'. Must be one of: {}",
                promise_type, VALID_PROMISE_TYPES.join(", ")
            )));
        }
        let amount: f64 = promised_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Promised amount must be a valid number".to_string(),
        ))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Promised amount must be positive".to_string(),
            ));
        }

        info!("Recording promise to pay of {} from customer {}", promised_amount, customer_id);

        self.repository.create_promise_to_pay(
            org_id, case_id, customer_id, customer_number, customer_name,
            promise_type, promised_amount, promise_date,
            installment_count, installment_frequency,
            related_invoice_ids, promised_by_name, promised_by_role,
            notes, recorded_by,
        ).await
    }

    /// Mark a promise as kept (fully paid)
    pub async fn keep_promise(&self, promise_id: Uuid, paid_amount: &str) -> AtlasResult<PromiseToPay> {
        let ptp = self.repository.get_promise_to_pay(promise_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Promise to pay {} not found", promise_id)
            ))?;

        if ptp.status != "pending" && ptp.status != "partially_kept" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot update promise in '{}' status", ptp.status)
            ));
        }

        let paid: f64 = paid_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Paid amount must be a valid number".to_string(),
        ))?;

        let promised: f64 = ptp.promised_amount.parse().unwrap_or(0.0);
        let already_paid: f64 = ptp.paid_amount.parse().unwrap_or(0.0);
        let total_paid = already_paid + paid;
        let remaining = (promised - total_paid).max(0.0);

        let new_status = if remaining <= 0.01 { "kept" } else { "partially_kept" };

        info!("Updating promise {} - paid {}, status: {}", promise_id, paid_amount, new_status);

        self.repository.update_promise_status(
            promise_id,
            new_status,
            Some(&format!("{:.2}", total_paid)),
            Some(&format!("{:.2}", remaining)),
            None,
            None,
        ).await
    }

    /// Mark a promise as broken
    pub async fn break_promise(&self, promise_id: Uuid, reason: Option<&str>) -> AtlasResult<PromiseToPay> {
        let ptp = self.repository.get_promise_to_pay(promise_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Promise to pay {} not found", promise_id)
            ))?;

        if ptp.status != "pending" && ptp.status != "partially_kept" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot break promise in '{}' status", ptp.status)
            ));
        }

        info!("Marking promise {} as broken", promise_id);

        self.repository.update_promise_status(
            promise_id,
            "broken",
            None,
            None,
            Some(chrono::Utc::now().date_naive()),
            reason,
        ).await
    }

    /// List promises to pay
    pub async fn list_promises_to_pay(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PromiseToPay>> {
        if let Some(s) = status {
            if !VALID_PROMISE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_PROMISE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_promises_to_pay(org_id, customer_id, status).await
    }

    // ========================================================================
    // Write-Off Management
    // ========================================================================

    /// Create a write-off request
    pub async fn create_write_off_request(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        write_off_type: &str,
        write_off_amount: &str,
        write_off_account_code: Option<&str>,
        reason: &str,
        related_invoice_ids: serde_json::Value,
        case_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WriteOffRequest> {
        if !VALID_WRITE_OFF_TYPES.contains(&write_off_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid write-off type '{}'. Must be one of: {}",
                write_off_type, VALID_WRITE_OFF_TYPES.join(", ")
            )));
        }
        let amount: f64 = write_off_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Write-off amount must be a valid number".to_string(),
        ))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Write-off amount must be positive".to_string(),
            ));
        }
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Write-off reason is required".to_string(),
            ));
        }

        let request_number = format!("WO-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating write-off request {} for customer {}", request_number, customer_id);

        self.repository.create_write_off_request(
            org_id, &request_number, customer_id, customer_number, customer_name,
            write_off_type, write_off_amount, write_off_account_code,
            reason, related_invoice_ids, case_id, created_by,
        ).await
    }

    /// Submit a write-off request for approval
    pub async fn submit_write_off(&self, request_id: Uuid, submitted_by: Uuid) -> AtlasResult<WriteOffRequest> {
        let wo = self.repository.get_write_off_request(request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Write-off request {} not found", request_id)
            ))?;

        if wo.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit write-off in '{}' status. Must be 'draft'.", wo.status)
            ));
        }

        info!("Submitting write-off request {}", wo.request_number);
        self.repository.update_write_off_status(request_id, "submitted", Some(submitted_by), None, None, None).await
    }

    /// Approve a write-off request
    pub async fn approve_write_off(&self, request_id: Uuid, approved_by: Uuid) -> AtlasResult<WriteOffRequest> {
        let wo = self.repository.get_write_off_request(request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Write-off request {} not found", request_id)
            ))?;

        if wo.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve write-off in '{}' status. Must be 'submitted'.", wo.status)
            ));
        }

        info!("Approving write-off request {}", wo.request_number);
        self.repository.update_write_off_status(request_id, "approved", None, Some(approved_by), None, None).await
    }

    /// Reject a write-off request
    pub async fn reject_write_off(&self, request_id: Uuid, reason: &str) -> AtlasResult<WriteOffRequest> {
        let wo = self.repository.get_write_off_request(request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Write-off request {} not found", request_id)
            ))?;

        if wo.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject write-off in '{}' status. Must be 'submitted'.", wo.status)
            ));
        }

        info!("Rejecting write-off request {}", wo.request_number);
        self.repository.update_write_off_status(request_id, "rejected", None, None, Some(reason), None).await
    }

    // ========================================================================
    // Aging Analysis
    // ========================================================================

    /// Calculate aging summary across all customers for a given date
    /// This returns a summary report combining all individual snapshots
    pub fn calculate_aging_summary(
        &self,
        snapshots: &[atlas_shared::ReceivablesAgingSnapshot],
        org_id: Uuid,
        as_of_date: chrono::NaiveDate,
    ) -> AgingSummary {
        let mut total_outstanding = 0.0_f64;
        let mut aging_current = 0.0;
        let mut aging_1_30 = 0.0;
        let mut aging_31_60 = 0.0;
        let mut aging_61_90 = 0.0;
        let mut aging_91_120 = 0.0;
        let mut aging_121_plus = 0.0;
        let mut overdue_count = 0;

        for snap in snapshots {
            total_outstanding += snap.total_outstanding.parse().unwrap_or(0.0);
            aging_current += snap.aging_current.parse().unwrap_or(0.0);
            aging_1_30 += snap.aging_1_30.parse().unwrap_or(0.0);
            aging_31_60 += snap.aging_31_60.parse().unwrap_or(0.0);
            aging_61_90 += snap.aging_61_90.parse().unwrap_or(0.0);
            aging_91_120 += snap.aging_91_120.parse().unwrap_or(0.0);
            aging_121_plus += snap.aging_121_plus.parse().unwrap_or(0.0);

            let overdue = snap.aging_1_30.parse::<f64>().unwrap_or(0.0)
                + snap.aging_31_60.parse::<f64>().unwrap_or(0.0)
                + snap.aging_61_90.parse::<f64>().unwrap_or(0.0)
                + snap.aging_91_120.parse::<f64>().unwrap_or(0.0)
                + snap.aging_121_plus.parse::<f64>().unwrap_or(0.0);
            if overdue > 0.0 {
                overdue_count += 1;
            }
        }

        let total_overdue = aging_1_30 + aging_31_60 + aging_61_90 + aging_91_120 + aging_121_plus;

        // Weighted average days overdue (approximate using bucket midpoints)
        let weighted_days = if total_overdue > 0.0 {
            (aging_1_30 * 15.0 + aging_31_60 * 45.0 + aging_61_90 * 75.0
                + aging_91_120 * 105.0 + aging_121_plus * 150.0) / total_overdue
        } else {
            0.0
        };

        AgingSummary {
            organization_id: org_id,
            as_of_date,
            total_outstanding: format!("{:.2}", total_outstanding),
            total_overdue: format!("{:.2}", total_overdue),
            aging_current: format!("{:.2}", aging_current),
            aging_1_30: format!("{:.2}", aging_1_30),
            aging_31_60: format!("{:.2}", aging_31_60),
            aging_61_90: format!("{:.2}", aging_61_90),
            aging_91_120: format!("{:.2}", aging_91_120),
            aging_121_plus: format!("{:.2}", aging_121_plus),
            customer_count: snapshots.len() as i32,
            overdue_customer_count: overdue_count,
            weighted_average_days_overdue: format!("{:.1}", weighted_days),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_risk_classifications() {
        assert!(VALID_RISK_CLASSIFICATIONS.contains(&"low"));
        assert!(VALID_RISK_CLASSIFICATIONS.contains(&"medium"));
        assert!(VALID_RISK_CLASSIFICATIONS.contains(&"high"));
        assert!(VALID_RISK_CLASSIFICATIONS.contains(&"very_high"));
        assert!(VALID_RISK_CLASSIFICATIONS.contains(&"defaulted"));
    }

    #[test]
    fn test_valid_case_types() {
        assert!(VALID_CASE_TYPES.contains(&"collection"));
        assert!(VALID_CASE_TYPES.contains(&"dispute"));
        assert!(VALID_CASE_TYPES.contains(&"bankruptcy"));
        assert!(VALID_CASE_TYPES.contains(&"skip_trace"));
    }

    #[test]
    fn test_valid_case_statuses() {
        assert!(VALID_CASE_STATUSES.contains(&"open"));
        assert!(VALID_CASE_STATUSES.contains(&"in_progress"));
        assert!(VALID_CASE_STATUSES.contains(&"resolved"));
        assert!(VALID_CASE_STATUSES.contains(&"closed"));
        assert!(VALID_CASE_STATUSES.contains(&"escalated"));
        assert!(VALID_CASE_STATUSES.contains(&"written_off"));
    }

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"medium"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"critical"));
    }

    #[test]
    fn test_valid_interaction_types() {
        assert!(VALID_INTERACTION_TYPES.contains(&"phone_call"));
        assert!(VALID_INTERACTION_TYPES.contains(&"email"));
        assert!(VALID_INTERACTION_TYPES.contains(&"letter"));
        assert!(VALID_INTERACTION_TYPES.contains(&"meeting"));
        assert!(VALID_INTERACTION_TYPES.contains(&"note"));
        assert!(VALID_INTERACTION_TYPES.contains(&"sms"));
    }

    #[test]
    fn test_valid_outcomes() {
        assert!(VALID_OUTCOMES.contains(&"contacted"));
        assert!(VALID_OUTCOMES.contains(&"promised_to_pay"));
        assert!(VALID_OUTCOMES.contains(&"disputed"));
        assert!(VALID_OUTCOMES.contains(&"agreed_payment_plan"));
    }

    #[test]
    fn test_valid_promise_types() {
        assert!(VALID_PROMISE_TYPES.contains(&"single_payment"));
        assert!(VALID_PROMISE_TYPES.contains(&"installment"));
        assert!(VALID_PROMISE_TYPES.contains(&"full_balance"));
    }

    #[test]
    fn test_valid_dunning_levels() {
        assert!(VALID_DUNNING_LEVELS.contains(&"reminder"));
        assert!(VALID_DUNNING_LEVELS.contains(&"first_notice"));
        assert!(VALID_DUNNING_LEVELS.contains(&"final_notice"));
        assert!(VALID_DUNNING_LEVELS.contains(&"pre_legal"));
        assert!(VALID_DUNNING_LEVELS.contains(&"legal"));
    }

    #[test]
    fn test_valid_write_off_types() {
        assert!(VALID_WRITE_OFF_TYPES.contains(&"bad_debt"));
        assert!(VALID_WRITE_OFF_TYPES.contains(&"small_balance"));
        assert!(VALID_WRITE_OFF_TYPES.contains(&"dispute"));
        assert!(VALID_WRITE_OFF_TYPES.contains(&"adjustment"));
    }

    #[test]
    fn test_aging_summary_calculation() {
        let engine = CollectionsEngine::new(Arc::new(crate::MockCollectionsRepository));

        let snapshots = vec![
            atlas_shared::ReceivablesAgingSnapshot {
                id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                snapshot_date: chrono::Utc::now().date_naive(),
                customer_id: Uuid::new_v4(),
                customer_number: Some("C001".to_string()),
                customer_name: Some("Customer 1".to_string()),
                total_outstanding: "10000.00".to_string(),
                aging_current: "5000.00".to_string(),
                aging_1_30: "3000.00".to_string(),
                aging_31_60: "1000.00".to_string(),
                aging_61_90: "500.00".to_string(),
                aging_91_120: "300.00".to_string(),
                aging_121_plus: "200.00".to_string(),
                count_current: 2,
                count_1_30: 1,
                count_31_60: 1,
                count_61_90: 1,
                count_91_120: 1,
                count_121_plus: 1,
                weighted_average_days_overdue: Some("35.0".to_string()),
                overdue_percent: Some("50.0".to_string()),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            atlas_shared::ReceivablesAgingSnapshot {
                id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                snapshot_date: chrono::Utc::now().date_naive(),
                customer_id: Uuid::new_v4(),
                customer_number: Some("C002".to_string()),
                customer_name: Some("Customer 2".to_string()),
                total_outstanding: "5000.00".to_string(),
                aging_current: "5000.00".to_string(),
                aging_1_30: "0.00".to_string(),
                aging_31_60: "0.00".to_string(),
                aging_61_90: "0.00".to_string(),
                aging_91_120: "0.00".to_string(),
                aging_121_plus: "0.00".to_string(),
                count_current: 3,
                count_1_30: 0,
                count_31_60: 0,
                count_61_90: 0,
                count_91_120: 0,
                count_121_plus: 0,
                weighted_average_days_overdue: Some("0.0".to_string()),
                overdue_percent: Some("0.0".to_string()),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
        ];

        let org_id = Uuid::new_v4();
        let as_of = chrono::Utc::now().date_naive();
        let summary = engine.calculate_aging_summary(&snapshots, org_id, as_of);

        assert_eq!(summary.customer_count, 2);
        assert_eq!(summary.overdue_customer_count, 1); // Only C001 has overdue
        assert!((summary.total_outstanding.parse::<f64>().unwrap() - 15000.0).abs() < 0.01);
        assert!((summary.aging_current.parse::<f64>().unwrap() - 10000.0).abs() < 0.01);
        assert!((summary.aging_1_30.parse::<f64>().unwrap() - 3000.0).abs() < 0.01);
        assert!((summary.total_overdue.parse::<f64>().unwrap() - 5000.0).abs() < 0.01);
    }

    #[test]
    fn test_aging_summary_empty() {
        let engine = CollectionsEngine::new(Arc::new(crate::MockCollectionsRepository));

        let org_id = Uuid::new_v4();
        let as_of = chrono::Utc::now().date_naive();
        let summary = engine.calculate_aging_summary(&[], org_id, as_of);

        assert_eq!(summary.customer_count, 0);
        assert_eq!(summary.overdue_customer_count, 0);
        assert!((summary.total_outstanding.parse::<f64>().unwrap()).abs() < 0.01);
        assert!((summary.total_overdue.parse::<f64>().unwrap()).abs() < 0.01);
    }
}
