//! Procurement Sourcing Engine
//!
//! Manages the full sourcing lifecycle: event creation, supplier invitation,
//! bid submission, scoring & evaluation, and award.
//!
//! Oracle Fusion Cloud ERP equivalent: Procurement > Sourcing > Negotiations

use atlas_shared::{
    SourcingEvent, SourcingEventLine, SourcingInvite,
    SupplierResponse, SupplierResponseLine,
    ScoringCriterion, ResponseScore,
    SourcingAward, SourcingAwardLine,
    SourcingTemplate, SourcingSummary,
    AtlasError, AtlasResult,
};
use super::SourcingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid event types
#[allow(dead_code)]
const VALID_EVENT_TYPES: &[&str] = &[
    "rfq", "rfp", "rfi", "auction",
];

/// Valid event statuses
#[allow(dead_code)]
const VALID_EVENT_STATUSES: &[&str] = &[
    "draft", "published", "response_open", "evaluation", "awarded", "cancelled", "closed",
];

/// Valid event styles
#[allow(dead_code)]
const VALID_EVENT_STYLES: &[&str] = &[
    "sealed", "open", "reverse_auction",
];

/// Valid scoring methods
#[allow(dead_code)]
const VALID_SCORING_METHODS: &[&str] = &[
    "weighted", "pass_fail", "manual", "lowest_price",
];

/// Valid response statuses
#[allow(dead_code)]
const VALID_RESPONSE_STATUSES: &[&str] = &[
    "draft", "submitted", "under_review", "shortlisted", "rejected", "awarded", "disqualified",
];

/// Valid award statuses
#[allow(dead_code)]
const VALID_AWARD_STATUSES: &[&str] = &[
    "pending", "approved", "rejected", "cancelled",
];

/// Valid award methods
#[allow(dead_code)]
const VALID_AWARD_METHODS: &[&str] = &[
    "single", "split", "best_value", "lowest_price",
];

/// Valid criterion types
#[allow(dead_code)]
const VALID_CRITERION_TYPES: &[&str] = &[
    "price", "quality", "delivery", "technical", "compliance", "custom",
];

/// Procurement Sourcing Engine
pub struct SourcingEngine {
    repository: Arc<dyn SourcingRepository>,
}

impl SourcingEngine {
    pub fn new(repository: Arc<dyn SourcingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Sourcing Events
    // ========================================================================

    /// Create a new sourcing event
    pub async fn create_event(
        &self,
        org_id: Uuid,
        title: &str,
        description: Option<&str>,
        event_type: &str,
        style: &str,
        response_deadline: chrono::NaiveDate,
        currency_code: &str,
        scoring_method: &str,
        template_id: Option<Uuid>,
        evaluation_lead_id: Option<Uuid>,
        evaluation_lead_name: Option<&str>,
        contact_person_id: Option<Uuid>,
        contact_person_name: Option<&str>,
        are_bids_visible: bool,
        allow_supplier_rank_visibility: bool,
        terms_and_conditions: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingEvent> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Sourcing event title is required".to_string(),
            ));
        }
        if !VALID_EVENT_TYPES.contains(&event_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid event type '{}'. Must be one of: {}",
                event_type, VALID_EVENT_TYPES.join(", ")
            )));
        }
        if !VALID_EVENT_STYLES.contains(&style) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid style '{}'. Must be one of: {}",
                style, VALID_EVENT_STYLES.join(", ")
            )));
        }
        if !VALID_SCORING_METHODS.contains(&scoring_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid scoring method '{}'. Must be one of: {}",
                scoring_method, VALID_SCORING_METHODS.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        let event_number = format!("SE-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating sourcing event {} ({})", event_number, title);

        self.repository.create_event(
            org_id, &event_number, title, description, event_type, style,
            response_deadline, currency_code, scoring_method, template_id,
            evaluation_lead_id, evaluation_lead_name,
            contact_person_id, contact_person_name,
            are_bids_visible, allow_supplier_rank_visibility,
            terms_and_conditions, created_by,
        ).await
    }

    /// Get a sourcing event by ID
    pub async fn get_event(&self, id: Uuid) -> AtlasResult<Option<SourcingEvent>> {
        self.repository.get_event(id).await
    }

    /// Get a sourcing event by event number
    pub async fn get_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<SourcingEvent>> {
        self.repository.get_event_by_number(org_id, event_number).await
    }

    /// List sourcing events with optional filters
    pub async fn list_events(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        event_type: Option<&str>,
    ) -> AtlasResult<Vec<SourcingEvent>> {
        if let Some(s) = status {
            if !VALID_EVENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_EVENT_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = event_type {
            if !VALID_EVENT_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid event type '{}'. Must be one of: {}",
                    t, VALID_EVENT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_events(org_id, status, event_type).await
    }

    /// Publish a sourcing event (moves from draft to published)
    pub async fn publish_event(&self, event_id: Uuid, published_by: Option<Uuid>) -> AtlasResult<SourcingEvent> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot publish event in '{}' status. Must be 'draft'.",
                event.status
            )));
        }

        // Verify event has at least one line
        let lines = self.repository.list_event_lines(event_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot publish sourcing event without at least one line".to_string(),
            ));
        }

        info!("Publishing sourcing event {}", event.event_number);
        self.repository.update_event_status(event_id, "published", published_by, None, None).await
    }

    /// Close a sourcing event for responses
    pub async fn close_event(&self, event_id: Uuid, closed_by: Option<Uuid>) -> AtlasResult<SourcingEvent> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status != "published" && event.status != "response_open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close event in '{}' status. Must be 'published' or 'response_open'.",
                event.status
            )));
        }

        info!("Closing sourcing event {}", event.event_number);
        self.repository.update_event_status(event_id, "evaluation", None, closed_by, None).await
    }

    /// Cancel a sourcing event
    pub async fn cancel_event(
        &self,
        event_id: Uuid,
        cancelled_by: Option<Uuid>,
        _reason: Option<&str>,
    ) -> AtlasResult<SourcingEvent> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status == "awarded" || event.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel event in '{}' status.",
                event.status
            )));
        }

        info!("Cancelling sourcing event {}", event.event_number);
        self.repository.update_event_status(event_id, "cancelled", None, None, cancelled_by).await
    }

    // ========================================================================
    // Sourcing Event Lines
    // ========================================================================

    /// Add a line to a sourcing event
    pub async fn add_event_line(
        &self,
        org_id: Uuid,
        event_id: Uuid,
        description: &str,
        item_number: Option<&str>,
        category: Option<&str>,
        quantity: &str,
        uom: &str,
        target_price: Option<&str>,
        target_total: Option<&str>,
        need_by_date: Option<chrono::NaiveDate>,
        ship_to: Option<&str>,
        specifications: Option<serde_json::Value>,
        allow_partial_quantity: bool,
        min_award_quantity: Option<&str>,
    ) -> AtlasResult<SourcingEventLine> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Lines can only be added to events in 'draft' status".to_string(),
            ));
        }

        if description.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Line description is required".to_string(),
            ));
        }

        let _qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;

        if let Some(tp) = target_price {
            let _: f64 = tp.parse().map_err(|_| AtlasError::ValidationFailed(
                "Target price must be a valid number".to_string(),
            ))?;
        }

        // Get next line number
        let existing_lines = self.repository.list_event_lines(event_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        info!("Adding line {} to sourcing event {}", line_number, event.event_number);

        self.repository.create_event_line(
            org_id, event_id, line_number, description, item_number, category,
            quantity, uom, target_price, target_total, need_by_date, ship_to,
            specifications, allow_partial_quantity, min_award_quantity,
        ).await
    }

    /// List lines for a sourcing event
    pub async fn list_event_lines(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingEventLine>> {
        self.repository.list_event_lines(event_id).await
    }

    // ========================================================================
    // Supplier Invitations
    // ========================================================================

    /// Invite a supplier to a sourcing event
    pub async fn invite_supplier(
        &self,
        org_id: Uuid,
        event_id: Uuid,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        supplier_email: Option<&str>,
    ) -> AtlasResult<SourcingInvite> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status == "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot invite suppliers to a draft event. Publish first.".to_string(),
            ));
        }

        // Check for duplicate invite
        let existing = self.repository.get_invite(event_id, supplier_id).await?;
        if existing.is_some() {
            return Err(AtlasError::Conflict(
                format!("Supplier {} already invited to this event", supplier_id)
            ));
        }

        info!("Inviting supplier {} to event {}", supplier_id, event.event_number);

        let invite = self.repository.create_invite(
            org_id, event_id, supplier_id, supplier_name, supplier_email,
        ).await?;

        // Update invitee count
        let invites = self.repository.list_invites(event_id).await?;
        self.repository.update_event_invite_count(event_id, invites.len() as i32).await?;

        Ok(invite)
    }

    /// List invites for a sourcing event
    pub async fn list_invites(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingInvite>> {
        self.repository.list_invites(event_id).await
    }

    // ========================================================================
    // Supplier Responses (Bids)
    // ========================================================================

    /// Submit a supplier response (bid)
    pub async fn submit_response(
        &self,
        org_id: Uuid,
        event_id: Uuid,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        cover_letter: Option<&str>,
        valid_until: Option<chrono::NaiveDate>,
        payment_terms: Option<&str>,
        lead_time_days: Option<i32>,
        warranty_months: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierResponse> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status != "published" && event.status != "response_open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit response to event in '{}' status.",
                event.status
            )));
        }

        // Verify supplier is invited
        let invite = self.repository.get_invite(event_id, supplier_id).await?
            .ok_or_else(|| AtlasError::ValidationFailed(
                format!("Supplier {} is not invited to this event", supplier_id)
            ))?;

        if invite.status == "disqualified" {
            return Err(AtlasError::ValidationFailed(
                "Supplier has been disqualified from this event".to_string(),
            ));
        }

        let response_number = format!("SR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Supplier {} submitting response {} to event {}", supplier_id, response_number, event.event_number);

        let response = self.repository.create_response(
            org_id, event_id, &response_number, supplier_id, supplier_name,
            cover_letter, valid_until, payment_terms,
            lead_time_days, warranty_months, created_by,
        ).await?;

        // Update invite status
        self.repository.update_invite_status(invite.id, "responded", None).await?;

        // Update event response count
        let responses = self.repository.list_responses(event_id, None).await?;
        self.repository.update_event_response_count(event_id, responses.len() as i32).await?;

        Ok(response)
    }

    /// Get a supplier response by ID
    pub async fn get_response(&self, id: Uuid) -> AtlasResult<Option<SupplierResponse>> {
        self.repository.get_response(id).await
    }

    /// List responses for a sourcing event
    pub async fn list_responses(
        &self,
        event_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<SupplierResponse>> {
        if let Some(s) = status {
            if !VALID_RESPONSE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid response status '{}'. Must be one of: {}",
                    s, VALID_RESPONSE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_responses(event_id, status).await
    }

    /// Add a line to a supplier response
    pub async fn add_response_line(
        &self,
        org_id: Uuid,
        response_id: Uuid,
        event_line_id: Uuid,
        unit_price: &str,
        quantity: &str,
        discount_percent: Option<&str>,
        promised_delivery_date: Option<chrono::NaiveDate>,
        lead_time_days: Option<i32>,
        supplier_notes: Option<&str>,
    ) -> AtlasResult<SupplierResponseLine> {
        let response = self.repository.get_response(response_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Supplier response {} not found", response_id)
            ))?;

        if response.status != "draft" && response.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                "Cannot add lines to a response that is not in draft or submitted status".to_string(),
            ));
        }

        let price: f64 = unit_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unit price must be a valid number".to_string(),
        ))?;
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;

        let line_amount = price * qty;

        let effective_price = if let Some(dp) = discount_percent {
            let dp_val: f64 = dp.parse().map_err(|_| AtlasError::ValidationFailed(
                "Discount percent must be a valid number".to_string(),
            ))?;
            Some(format!("{:.2}", price * (1.0 - dp_val / 100.0)))
        } else {
            None
        };

        // Get next line number
        let existing_lines = self.repository.list_response_lines(response_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        let line = self.repository.create_response_line(
            org_id, response_id, event_line_id, line_number,
            unit_price, quantity, &format!("{:.2}", line_amount),
            discount_percent, effective_price.as_deref(),
            promised_delivery_date, lead_time_days, supplier_notes,
        ).await?;

        // Update response total
        let all_lines = self.repository.list_response_lines(response_id).await?;
        let total: f64 = all_lines.iter()
            .map(|l| l.line_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        self.repository.update_response_total(response_id, &format!("{:.2}", total)).await?;

        Ok(line)
    }

    /// List response lines for a supplier response
    pub async fn list_response_lines(&self, response_id: Uuid) -> AtlasResult<Vec<SupplierResponseLine>> {
        self.repository.list_response_lines(response_id).await
    }

    // ========================================================================
    // Scoring & Evaluation
    // ========================================================================

    /// Add a scoring criterion to an event
    pub async fn add_scoring_criterion(
        &self,
        org_id: Uuid,
        event_id: Uuid,
        name: &str,
        description: Option<&str>,
        weight: &str,
        max_score: &str,
        criterion_type: &str,
        display_order: i32,
        is_mandatory: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ScoringCriterion> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status == "awarded" || event.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                "Cannot modify scoring criteria for awarded or cancelled events".to_string(),
            ));
        }

        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Criterion name is required".to_string(),
            ));
        }

        let weight_val: f64 = weight.parse().map_err(|_| AtlasError::ValidationFailed(
            "Weight must be a valid number".to_string(),
        ))?;
        if weight_val <= 0.0 || weight_val > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "Weight must be between 0 and 100".to_string(),
            ));
        }

        let _max: f64 = max_score.parse().map_err(|_| AtlasError::ValidationFailed(
            "Max score must be a valid number".to_string(),
        ))?;

        if !VALID_CRITERION_TYPES.contains(&criterion_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid criterion type '{}'. Must be one of: {}",
                criterion_type, VALID_CRITERION_TYPES.join(", ")
            )));
        }

        info!("Adding scoring criterion '{}' to event {}", name, event.event_number);

        self.repository.create_scoring_criterion(
            org_id, event_id, name, description, weight, max_score,
            criterion_type, display_order, is_mandatory, created_by,
        ).await
    }

    /// List scoring criteria for an event
    pub async fn list_scoring_criteria(&self, event_id: Uuid) -> AtlasResult<Vec<ScoringCriterion>> {
        self.repository.list_scoring_criteria(event_id).await
    }

    /// Score a response for a specific criterion
    pub async fn score_response(
        &self,
        org_id: Uuid,
        response_id: Uuid,
        criterion_id: Uuid,
        score: &str,
        notes: Option<&str>,
        scored_by: Option<Uuid>,
    ) -> AtlasResult<ResponseScore> {
        let response = self.repository.get_response(response_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Response {} not found", response_id)
            ))?;

        if response.status != "submitted" && response.status != "under_review" {
            return Err(AtlasError::WorkflowError(
                "Can only score submitted or under-review responses".to_string(),
            ));
        }

        let criterion = self.repository.get_scoring_criterion(criterion_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Scoring criterion {} not found", criterion_id)
            ))?;

        let score_val: f64 = score.parse().map_err(|_| AtlasError::ValidationFailed(
            "Score must be a valid number".to_string(),
        ))?;

        let max_score: f64 = criterion.max_score.parse().unwrap_or(100.0);
        if score_val < 0.0 || score_val > max_score {
            return Err(AtlasError::ValidationFailed(format!(
                "Score must be between 0 and {}", max_score
            )));
        }

        let weight: f64 = criterion.weight.parse().unwrap_or(0.0);
        let weighted_score = score_val * weight / 100.0;

        info!("Scoring response {} criterion {} with {} (weighted: {:.2})",
            response_id, criterion_id, score_val, weighted_score);

        // Move response to under_review if it's still submitted
        if response.status == "submitted" {
            self.repository.update_response_status(response_id, "under_review").await?;
        }

        self.repository.upsert_response_score(
            org_id, response_id, criterion_id, score,
            &format!("{:.2}", weighted_score), notes, scored_by,
        ).await
    }

    /// List scores for a response
    pub async fn list_response_scores(&self, response_id: Uuid) -> AtlasResult<Vec<ResponseScore>> {
        self.repository.list_response_scores(response_id).await
    }

    /// Evaluate all responses for an event (calculate total scores and ranks)
    pub async fn evaluate_responses(&self, event_id: Uuid, evaluated_by: Option<Uuid>) -> AtlasResult<Vec<SupplierResponse>> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status != "evaluation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot evaluate responses for event in '{}' status. Must be in 'evaluation'.",
                event.status
            )));
        }

        let responses = self.repository.list_responses(event_id, None).await?;
        let criteria = self.repository.list_scoring_criteria(event_id).await?;

        if criteria.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot evaluate responses without scoring criteria".to_string(),
            ));
        }

        let mut scored_responses: Vec<(Uuid, f64)> = Vec::new();

        for response in &responses {
            if response.status == "disqualified" {
                continue;
            }

            let scores = self.repository.list_response_scores(response.id).await?;
            let total_weighted_score: f64 = scores.iter()
                .map(|s| s.weighted_score.parse::<f64>().unwrap_or(0.0))
                .sum();

            self.repository.update_response_score_total(
                response.id,
                &format!("{:.2}", total_weighted_score),
                evaluated_by,
            ).await?;

            scored_responses.push((response.id, total_weighted_score));
        }

        // Sort by score descending and assign ranks
        scored_responses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, (response_id, _score)) in scored_responses.iter().enumerate() {
            self.repository.update_response_rank(*response_id, (rank + 1) as i32).await?;
        }

        info!("Evaluated {} responses for event {}", responses.len(), event.event_number);

        // Return updated responses
        self.repository.list_responses(event_id, None).await
    }

    // ========================================================================
    // Award Management
    // ========================================================================

    /// Create an award for a sourcing event
    pub async fn create_award(
        &self,
        org_id: Uuid,
        event_id: Uuid,
        award_method: &str,
        award_lines: &[SourcingAwardLineRequest],
        award_rationale: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingAward> {
        let event = self.repository.get_event(event_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Sourcing event {} not found", event_id)
            ))?;

        if event.status != "evaluation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot create award for event in '{}' status. Must be in 'evaluation'.",
                event.status
            )));
        }

        if !VALID_AWARD_METHODS.contains(&award_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid award method '{}'. Must be one of: {}",
                award_method, VALID_AWARD_METHODS.join(", ")
            )));
        }

        if award_lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Award must have at least one line".to_string(),
            ));
        }

        let award_number = format!("AW-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        // Calculate total awarded amount
        let total: f64 = award_lines.iter()
            .map(|l| l.awarded_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        info!("Creating award {} for event {} (method: {})", award_number, event.event_number, award_method);

        let award = self.repository.create_award(
            org_id, event_id, &award_number, award_method,
            &format!("{:.2}", total), award_rationale, created_by,
        ).await?;

        // Create award lines
        for line in award_lines {
            self.repository.create_award_line(
                org_id, award.id, line.event_line_id, line.response_id,
                line.supplier_id, line.supplier_name.as_deref(),
                &line.awarded_quantity, &line.awarded_unit_price, &line.awarded_amount,
            ).await?;
        }

        // Update award with lines JSON
        let award_lines_data = self.repository.list_award_lines(award.id).await?;
        let total_awarded: f64 = award_lines_data.iter()
            .map(|l| l.awarded_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Update event line awards
        for al in &award_lines_data {
            self.repository.update_event_line_award(
                al.event_line_id, al.supplier_id,
                al.supplier_name.as_deref(),
                &al.awarded_unit_price, &al.awarded_quantity,
            ).await?;
        }

        // Build award summary for the event
        let summary = serde_json::json!({
            "award_id": award.id,
            "award_number": award_number,
            "award_method": award_method,
            "total_awarded": format!("{:.2}", total_awarded),
            "line_count": award_lines_data.len(),
        });

        self.repository.update_event_award_summary(event_id, summary).await?;

        let mut award = award;
        award.total_awarded_amount = format!("{:.2}", total_awarded);
        award.lines = serde_json::to_value(&award_lines_data).unwrap_or(serde_json::json!([]));

        Ok(award)
    }

    /// Get an award by ID
    pub async fn get_award(&self, id: Uuid) -> AtlasResult<Option<SourcingAward>> {
        self.repository.get_award(id).await
    }

    /// List awards for an event
    pub async fn list_awards(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingAward>> {
        self.repository.list_awards(event_id).await
    }

    /// Approve an award and finalize the sourcing event
    pub async fn approve_award(
        &self,
        award_id: Uuid,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<SourcingAward> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Award {} not found", award_id)
            ))?;

        if award.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve award in '{}' status. Must be 'pending'.",
                award.status
            )));
        }

        info!("Approving award {}", award.award_number);

        let updated = self.repository.update_award_status(
            award_id, "approved", approved_by, None,
        ).await?;

        // Update event status to awarded
        self.repository.update_event_status(award.event_id, "awarded", None, None, None).await?;

        // Update winning response statuses
        let award_lines = self.repository.list_award_lines(award_id).await?;
        for al in &award_lines {
            self.repository.update_response_status(al.response_id, "awarded").await?;
        }

        Ok(updated)
    }

    /// Reject an award
    pub async fn reject_award(
        &self,
        award_id: Uuid,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<SourcingAward> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Award {} not found", award_id)
            ))?;

        if award.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject award in '{}' status.", award.status
            )));
        }

        info!("Rejecting award {}", award.award_number);
        self.repository.update_award_status(award_id, "rejected", None, rejected_reason).await
    }

    /// List award lines for an award
    pub async fn list_award_lines(&self, award_id: Uuid) -> AtlasResult<Vec<SourcingAwardLine>> {
        self.repository.list_award_lines(award_id).await
    }

    // ========================================================================
    // Sourcing Templates
    // ========================================================================

    /// Create a sourcing template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        default_event_type: &str,
        default_style: &str,
        default_scoring_method: &str,
        default_response_deadline_days: i32,
        currency_code: &str,
        default_bids_visible: bool,
        default_terms: Option<&str>,
        default_scoring_criteria: serde_json::Value,
        default_lines: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingTemplate> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template code and name are required".to_string(),
            ));
        }
        if !VALID_EVENT_TYPES.contains(&default_event_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid default event type '{}'. Must be one of: {}",
                default_event_type, VALID_EVENT_TYPES.join(", ")
            )));
        }

        info!("Creating sourcing template '{}' for org {}", code, org_id);

        self.repository.create_template(
            org_id, code, name, description, default_event_type, default_style,
            default_scoring_method, default_response_deadline_days,
            currency_code, default_bids_visible, default_terms,
            default_scoring_criteria, default_lines, created_by,
        ).await
    }

    /// Get a template by code
    pub async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SourcingTemplate>> {
        self.repository.get_template(org_id, code).await
    }

    /// List templates
    pub async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<SourcingTemplate>> {
        self.repository.list_templates(org_id).await
    }

    /// Delete (soft-delete) a template
    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_template(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template '{}' not found", code)
            ))?;

        self.repository.delete_template(org_id, code).await
    }

    // ========================================================================
    // Dashboard Summary
    // ========================================================================

    /// Get sourcing dashboard summary
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<SourcingSummary> {
        let all_events = self.repository.list_events(org_id, None, None).await?;

        let active_count = all_events.iter()
            .filter(|e| e.status == "published" || e.status == "response_open" || e.status == "evaluation")
            .count() as i32;
        let draft_count = all_events.iter()
            .filter(|e| e.status == "draft")
            .count() as i32;
        let pending_eval = all_events.iter()
            .filter(|e| e.status == "evaluation")
            .count() as i32;
        let awarded_count = all_events.iter()
            .filter(|e| e.status == "awarded")
            .count() as i32;

        let total_awarded_value: f64 = all_events.iter()
            .filter(|e| e.status == "awarded")
            .map(|e| e.award_summary.get("total_awarded")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0))
            .sum();

        // Events by status
        let mut status_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for e in &all_events {
            *status_map.entry(e.status.clone()).or_insert(0) += 1;
        }
        let events_by_status: serde_json::Value = status_map.into_iter()
            .map(|(k, v)| serde_json::json!({"status": k, "count": v}))
            .collect();

        // Events by type
        let mut type_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for e in &all_events {
            *type_map.entry(e.event_type.clone()).or_insert(0) += 1;
        }
        let events_by_type: serde_json::Value = type_map.into_iter()
            .map(|(k, v)| serde_json::json!({"type": k, "count": v}))
            .collect();

        // Upcoming deadlines
        let today = chrono::Utc::now().date_naive();
        let upcoming: Vec<serde_json::Value> = all_events.iter()
            .filter(|e| e.status == "published" || e.status == "response_open")
            .filter(|e| e.response_deadline >= today)
            .take(5)
            .map(|e| serde_json::json!({
                "event_number": e.event_number,
                "title": e.title,
                "deadline": e.response_deadline,
                "response_count": e.response_count,
            }))
            .collect();

        Ok(SourcingSummary {
            active_event_count: active_count,
            draft_event_count: draft_count,
            pending_evaluation_count: pending_eval,
            awarded_event_count: awarded_count,
            total_awarded_value: format!("{:.2}", total_awarded_value),
            average_savings_percent: "0.00".to_string(), // Would need historical data
            events_by_status,
            events_by_type,
            top_suppliers: serde_json::json!([]),
            upcoming_deadlines: serde_json::Value::Array(upcoming),
        })
    }
}

/// Request to create an award line (used internally)
pub struct SourcingAwardLineRequest {
    pub event_line_id: Uuid,
    pub response_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub awarded_quantity: String,
    pub awarded_unit_price: String,
    pub awarded_amount: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_event_types() {
        assert!(VALID_EVENT_TYPES.contains(&"rfq"));
        assert!(VALID_EVENT_TYPES.contains(&"rfp"));
        assert!(VALID_EVENT_TYPES.contains(&"rfi"));
        assert!(VALID_EVENT_TYPES.contains(&"auction"));
    }

    #[test]
    fn test_valid_event_styles() {
        assert!(VALID_EVENT_STYLES.contains(&"sealed"));
        assert!(VALID_EVENT_STYLES.contains(&"open"));
        assert!(VALID_EVENT_STYLES.contains(&"reverse_auction"));
    }

    #[test]
    fn test_valid_scoring_methods() {
        assert!(VALID_SCORING_METHODS.contains(&"weighted"));
        assert!(VALID_SCORING_METHODS.contains(&"pass_fail"));
        assert!(VALID_SCORING_METHODS.contains(&"manual"));
        assert!(VALID_SCORING_METHODS.contains(&"lowest_price"));
    }

    #[test]
    fn test_valid_response_statuses() {
        assert!(VALID_RESPONSE_STATUSES.contains(&"draft"));
        assert!(VALID_RESPONSE_STATUSES.contains(&"submitted"));
        assert!(VALID_RESPONSE_STATUSES.contains(&"awarded"));
        assert!(VALID_RESPONSE_STATUSES.contains(&"disqualified"));
    }

    #[test]
    fn test_valid_award_methods() {
        assert!(VALID_AWARD_METHODS.contains(&"single"));
        assert!(VALID_AWARD_METHODS.contains(&"split"));
        assert!(VALID_AWARD_METHODS.contains(&"best_value"));
        assert!(VALID_AWARD_METHODS.contains(&"lowest_price"));
    }

    #[test]
    fn test_valid_criterion_types() {
        assert!(VALID_CRITERION_TYPES.contains(&"price"));
        assert!(VALID_CRITERION_TYPES.contains(&"quality"));
        assert!(VALID_CRITERION_TYPES.contains(&"delivery"));
        assert!(VALID_CRITERION_TYPES.contains(&"technical"));
    }
}
