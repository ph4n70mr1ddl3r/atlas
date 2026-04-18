//! Procurement Sourcing Repository
//!
//! PostgreSQL storage for sourcing events, lines, invites,
//! supplier responses, scoring, awards, and templates.

use atlas_shared::{
    SourcingEvent, SourcingEventLine, SourcingInvite,
    SupplierResponse, SupplierResponseLine,
    ScoringCriterion, ResponseScore,
    SourcingAward, SourcingAwardLine,
    SourcingTemplate,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_event(row: &sqlx::postgres::PgRow) -> SourcingEvent {
    SourcingEvent {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        event_number: row.get("event_number"),
        title: row.get("title"),
        description: row.get("description"),
        event_type: row.get("event_type"),
        status: row.get("status"),
        style: row.get("style"),
        response_deadline: row.get("response_deadline"),
        published_at: row.get("published_at"),
        closed_at: row.get("closed_at"),
        currency_code: row.get("currency_code"),
        template_id: row.get("template_id"),
        template_name: row.get("template_name"),
        evaluation_lead_id: row.get("evaluation_lead_id"),
        evaluation_lead_name: row.get("evaluation_lead_name"),
        scoring_method: row.get("scoring_method"),
        are_bids_visible: row.get("are_bids_visible"),
        allow_supplier_rank_visibility: row.get("allow_supplier_rank_visibility"),
        contact_person_id: row.get("contact_person_id"),
        contact_person_name: row.get("contact_person_name"),
        terms_and_conditions: row.get("terms_and_conditions"),
        attachments: row.try_get("attachments").unwrap_or(serde_json::json!([])),
        invited_supplier_count: row.get("invited_supplier_count"),
        response_count: row.get("response_count"),
        award_summary: row.try_get("award_summary").unwrap_or(serde_json::json!({})),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        cancelled_by: row.get("cancelled_by"),
        cancelled_at: row.get("cancelled_at"),
        cancellation_reason: row.get("cancellation_reason"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_event_line(row: &sqlx::postgres::PgRow) -> SourcingEventLine {
    SourcingEventLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        event_id: row.get("event_id"),
        line_number: row.get("line_number"),
        description: row.get("description"),
        item_number: row.get("item_number"),
        category: row.get("category"),
        quantity: get_num(row, "quantity"),
        uom: row.get("uom"),
        target_price: row.try_get("target_price").ok(),
        target_total: row.try_get("target_total").ok(),
        need_by_date: row.get("need_by_date"),
        ship_to: row.get("ship_to"),
        specifications: row.try_get("specifications").ok().flatten(),
        allow_partial_quantity: row.get("allow_partial_quantity"),
        min_award_quantity: row.try_get("min_award_quantity").ok(),
        status: row.get("status"),
        awarded_supplier_id: row.get("awarded_supplier_id"),
        awarded_supplier_name: row.get("awarded_supplier_name"),
        awarded_price: row.try_get("awarded_price").ok(),
        awarded_quantity: row.try_get("awarded_quantity").ok(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_invite(row: &sqlx::postgres::PgRow) -> SourcingInvite {
    SourcingInvite {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        event_id: row.get("event_id"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        supplier_email: row.get("supplier_email"),
        is_viewed: row.get("is_viewed"),
        viewed_at: row.get("viewed_at"),
        has_responded: row.get("has_responded"),
        responded_at: row.get("responded_at"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_response(row: &sqlx::postgres::PgRow) -> SupplierResponse {
    SupplierResponse {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        event_id: row.get("event_id"),
        response_number: row.get("response_number"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        status: row.get("status"),
        total_amount: get_num(row, "total_amount"),
        total_score: row.try_get("total_score").ok(),
        rank: row.try_get("rank").ok(),
        is_compliant: row.try_get("is_compliant").ok(),
        cover_letter: row.get("cover_letter"),
        valid_until: row.get("valid_until"),
        payment_terms: row.get("payment_terms"),
        lead_time_days: row.try_get("lead_time_days").ok(),
        warranty_months: row.try_get("warranty_months").ok(),
        attachments: row.try_get("attachments").unwrap_or(serde_json::json!([])),
        evaluation_notes: row.get("evaluation_notes"),
        submitted_at: row.get("submitted_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        evaluated_by: row.get("evaluated_by"),
        evaluated_at: row.get("evaluated_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_response_line(row: &sqlx::postgres::PgRow) -> SupplierResponseLine {
    SupplierResponseLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        response_id: row.get("response_id"),
        event_line_id: row.get("event_line_id"),
        line_number: row.get("line_number"),
        unit_price: get_num(row, "unit_price"),
        quantity: get_num(row, "quantity"),
        line_amount: get_num(row, "line_amount"),
        discount_percent: row.try_get("discount_percent").ok(),
        effective_price: row.try_get("effective_price").ok(),
        promised_delivery_date: row.get("promised_delivery_date"),
        lead_time_days: row.try_get("lead_time_days").ok(),
        is_compliant: row.try_get("is_compliant").ok(),
        score: row.try_get("score").ok(),
        supplier_notes: row.get("supplier_notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_criterion(row: &sqlx::postgres::PgRow) -> ScoringCriterion {
    ScoringCriterion {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        event_id: row.get("event_id"),
        name: row.get("name"),
        description: row.get("description"),
        weight: get_num(row, "weight"),
        max_score: get_num(row, "max_score"),
        criterion_type: row.get("criterion_type"),
        display_order: row.get("display_order"),
        is_mandatory: row.get("is_mandatory"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_response_score(row: &sqlx::postgres::PgRow) -> ResponseScore {
    ResponseScore {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        response_id: row.get("response_id"),
        criterion_id: row.get("criterion_id"),
        score: get_num(row, "score"),
        weighted_score: get_num(row, "weighted_score"),
        notes: row.get("notes"),
        scored_by: row.get("scored_by"),
        scored_at: row.get("scored_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_award(row: &sqlx::postgres::PgRow) -> SourcingAward {
    SourcingAward {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        event_id: row.get("event_id"),
        award_number: row.get("award_number"),
        status: row.get("status"),
        award_method: row.get("award_method"),
        total_awarded_amount: get_num(row, "total_awarded_amount"),
        award_rationale: row.get("award_rationale"),
        awarded_by: row.get("awarded_by"),
        awarded_at: row.get("awarded_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        rejected_reason: row.get("rejected_reason"),
        lines: row.try_get("lines").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_award_line(row: &sqlx::postgres::PgRow) -> SourcingAwardLine {
    SourcingAwardLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        award_id: row.get("award_id"),
        event_line_id: row.get("event_line_id"),
        response_id: row.get("response_id"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        awarded_quantity: get_num(row, "awarded_quantity"),
        awarded_unit_price: get_num(row, "awarded_unit_price"),
        awarded_amount: get_num(row, "awarded_amount"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> SourcingTemplate {
    SourcingTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        default_event_type: row.get("default_event_type"),
        default_style: row.get("default_style"),
        default_scoring_method: row.get("default_scoring_method"),
        default_response_deadline_days: row.get("default_response_deadline_days"),
        currency_code: row.get("currency_code"),
        default_bids_visible: row.get("default_bids_visible"),
        default_terms: row.get("default_terms"),
        default_scoring_criteria: row.try_get("default_scoring_criteria").unwrap_or(serde_json::json!([])),
        default_lines: row.try_get("default_lines").unwrap_or(serde_json::json!([])),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// Repository trait for sourcing data storage
#[async_trait]
pub trait SourcingRepository: Send + Sync {
    // Events
    async fn create_event(
        &self, org_id: Uuid, event_number: &str, title: &str,
        description: Option<&str>, event_type: &str, style: &str,
        response_deadline: chrono::NaiveDate, currency_code: &str,
        scoring_method: &str, template_id: Option<Uuid>,
        evaluation_lead_id: Option<Uuid>, evaluation_lead_name: Option<&str>,
        contact_person_id: Option<Uuid>, contact_person_name: Option<&str>,
        are_bids_visible: bool, allow_supplier_rank_visibility: bool,
        terms_and_conditions: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingEvent>;
    async fn get_event(&self, id: Uuid) -> AtlasResult<Option<SourcingEvent>>;
    async fn get_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<SourcingEvent>>;
    async fn list_events(&self, org_id: Uuid, status: Option<&str>, event_type: Option<&str>) -> AtlasResult<Vec<SourcingEvent>>;
    async fn update_event_status(&self, id: Uuid, status: &str, published_by: Option<Uuid>, closed_by: Option<Uuid>, cancelled_by: Option<Uuid>) -> AtlasResult<SourcingEvent>;
    async fn update_event_invite_count(&self, id: Uuid, count: i32) -> AtlasResult<()>;
    async fn update_event_response_count(&self, id: Uuid, count: i32) -> AtlasResult<()>;
    async fn update_event_award_summary(&self, id: Uuid, summary: serde_json::Value) -> AtlasResult<()>;

    // Event Lines
    async fn create_event_line(
        &self, org_id: Uuid, event_id: Uuid, line_number: i32,
        description: &str, item_number: Option<&str>, category: Option<&str>,
        quantity: &str, uom: &str, target_price: Option<&str>, target_total: Option<&str>,
        need_by_date: Option<chrono::NaiveDate>, ship_to: Option<&str>,
        specifications: Option<serde_json::Value>, allow_partial_quantity: bool,
        min_award_quantity: Option<&str>,
    ) -> AtlasResult<SourcingEventLine>;
    async fn list_event_lines(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingEventLine>>;
    async fn update_event_line_award(&self, line_id: Uuid, supplier_id: Uuid, supplier_name: Option<&str>, awarded_price: &str, awarded_quantity: &str) -> AtlasResult<()>;

    // Invites
    async fn create_invite(&self, org_id: Uuid, event_id: Uuid, supplier_id: Uuid, supplier_name: Option<&str>, supplier_email: Option<&str>) -> AtlasResult<SourcingInvite>;
    async fn get_invite(&self, event_id: Uuid, supplier_id: Uuid) -> AtlasResult<Option<SourcingInvite>>;
    async fn list_invites(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingInvite>>;
    async fn update_invite_status(&self, id: Uuid, status: &str, viewed_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<()>;

    // Responses
    async fn create_response(&self, org_id: Uuid, event_id: Uuid, response_number: &str, supplier_id: Uuid, supplier_name: Option<&str>, cover_letter: Option<&str>, valid_until: Option<chrono::NaiveDate>, payment_terms: Option<&str>, lead_time_days: Option<i32>, warranty_months: Option<i32>, created_by: Option<Uuid>) -> AtlasResult<SupplierResponse>;
    async fn get_response(&self, id: Uuid) -> AtlasResult<Option<SupplierResponse>>;
    async fn list_responses(&self, event_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SupplierResponse>>;
    async fn update_response_total(&self, id: Uuid, total: &str) -> AtlasResult<()>;
    async fn update_response_status(&self, id: Uuid, status: &str) -> AtlasResult<()>;
    async fn update_response_score_total(&self, id: Uuid, total_score: &str, evaluated_by: Option<Uuid>) -> AtlasResult<()>;
    async fn update_response_rank(&self, id: Uuid, rank: i32) -> AtlasResult<()>;

    // Response Lines
    async fn create_response_line(&self, org_id: Uuid, response_id: Uuid, event_line_id: Uuid, line_number: i32, unit_price: &str, quantity: &str, line_amount: &str, discount_percent: Option<&str>, effective_price: Option<&str>, promised_delivery_date: Option<chrono::NaiveDate>, lead_time_days: Option<i32>, supplier_notes: Option<&str>) -> AtlasResult<SupplierResponseLine>;
    async fn list_response_lines(&self, response_id: Uuid) -> AtlasResult<Vec<SupplierResponseLine>>;

    // Scoring
    async fn create_scoring_criterion(&self, org_id: Uuid, event_id: Uuid, name: &str, description: Option<&str>, weight: &str, max_score: &str, criterion_type: &str, display_order: i32, is_mandatory: bool, created_by: Option<Uuid>) -> AtlasResult<ScoringCriterion>;
    async fn get_scoring_criterion(&self, id: Uuid) -> AtlasResult<Option<ScoringCriterion>>;
    async fn list_scoring_criteria(&self, event_id: Uuid) -> AtlasResult<Vec<ScoringCriterion>>;
    async fn upsert_response_score(&self, org_id: Uuid, response_id: Uuid, criterion_id: Uuid, score: &str, weighted_score: &str, notes: Option<&str>, scored_by: Option<Uuid>) -> AtlasResult<ResponseScore>;
    async fn list_response_scores(&self, response_id: Uuid) -> AtlasResult<Vec<ResponseScore>>;

    // Awards
    async fn create_award(&self, org_id: Uuid, event_id: Uuid, award_number: &str, award_method: &str, total_awarded_amount: &str, award_rationale: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<SourcingAward>;
    async fn get_award(&self, id: Uuid) -> AtlasResult<Option<SourcingAward>>;
    async fn list_awards(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingAward>>;
    async fn update_award_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, rejected_reason: Option<&str>) -> AtlasResult<SourcingAward>;
    async fn create_award_line(&self, org_id: Uuid, award_id: Uuid, event_line_id: Uuid, response_id: Uuid, supplier_id: Uuid, supplier_name: Option<&str>, awarded_quantity: &str, awarded_unit_price: &str, awarded_amount: &str) -> AtlasResult<SourcingAwardLine>;
    async fn list_award_lines(&self, award_id: Uuid) -> AtlasResult<Vec<SourcingAwardLine>>;

    // Templates
    async fn create_template(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, default_event_type: &str, default_style: &str, default_scoring_method: &str, default_response_deadline_days: i32, currency_code: &str, default_bids_visible: bool, default_terms: Option<&str>, default_scoring_criteria: serde_json::Value, default_lines: serde_json::Value, created_by: Option<Uuid>) -> AtlasResult<SourcingTemplate>;
    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SourcingTemplate>>;
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<SourcingTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresSourcingRepository {
    pool: PgPool,
}

impl PostgresSourcingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SourcingRepository for PostgresSourcingRepository {
    // ========================================================================
    // Events
    // ========================================================================

    async fn create_event(
        &self, org_id: Uuid, event_number: &str, title: &str,
        description: Option<&str>, event_type: &str, style: &str,
        response_deadline: chrono::NaiveDate, currency_code: &str,
        scoring_method: &str, template_id: Option<Uuid>,
        evaluation_lead_id: Option<Uuid>, evaluation_lead_name: Option<&str>,
        contact_person_id: Option<Uuid>, contact_person_name: Option<&str>,
        are_bids_visible: bool, allow_supplier_rank_visibility: bool,
        terms_and_conditions: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingEvent> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sourcing_events
                (organization_id, event_number, title, description, event_type, style,
                 status, response_deadline, currency_code, scoring_method,
                 template_id, evaluation_lead_id, evaluation_lead_name,
                 contact_person_id, contact_person_name,
                 are_bids_visible, allow_supplier_rank_visibility,
                 terms_and_conditions, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7, $8, $9,
                    $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(event_number).bind(title).bind(description)
        .bind(event_type).bind(style)
        .bind(response_deadline).bind(currency_code).bind(scoring_method)
        .bind(template_id).bind(evaluation_lead_id).bind(evaluation_lead_name)
        .bind(contact_person_id).bind(contact_person_name)
        .bind(are_bids_visible).bind(allow_supplier_rank_visibility)
        .bind(terms_and_conditions).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_event(&row))
    }

    async fn get_event(&self, id: Uuid) -> AtlasResult<Option<SourcingEvent>> {
        let row = sqlx::query("SELECT * FROM _atlas.sourcing_events WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_event(&r)))
    }

    async fn get_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<SourcingEvent>> {
        let row = sqlx::query("SELECT * FROM _atlas.sourcing_events WHERE organization_id = $1 AND event_number = $2")
            .bind(org_id).bind(event_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_event(&r)))
    }

    async fn list_events(&self, org_id: Uuid, status: Option<&str>, event_type: Option<&str>) -> AtlasResult<Vec<SourcingEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.sourcing_events
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR event_type = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_event(&r)).collect())
    }

    async fn update_event_status(&self, id: Uuid, status: &str, published_by: Option<Uuid>, closed_by: Option<Uuid>, cancelled_by: Option<Uuid>) -> AtlasResult<SourcingEvent> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.sourcing_events
            SET status = $2,
                published_at = CASE WHEN $2 = 'published' THEN now() ELSE published_at END,
                closed_at = CASE WHEN $2 IN ('evaluation', 'awarded', 'closed') THEN now() ELSE closed_at END,
                cancelled_by = $5,
                cancelled_at = CASE WHEN $2 = 'cancelled' THEN now() ELSE cancelled_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(published_by).bind(closed_by).bind(cancelled_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_event(&row))
    }

    async fn update_event_invite_count(&self, id: Uuid, count: i32) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.sourcing_events SET invited_supplier_count = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(count)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_event_response_count(&self, id: Uuid, count: i32) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.sourcing_events SET response_count = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(count)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_event_award_summary(&self, id: Uuid, summary: serde_json::Value) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.sourcing_events SET award_summary = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(summary)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Event Lines
    // ========================================================================

    async fn create_event_line(
        &self, org_id: Uuid, event_id: Uuid, line_number: i32,
        description: &str, item_number: Option<&str>, category: Option<&str>,
        quantity: &str, uom: &str, target_price: Option<&str>, target_total: Option<&str>,
        need_by_date: Option<chrono::NaiveDate>, ship_to: Option<&str>,
        specifications: Option<serde_json::Value>, allow_partial_quantity: bool,
        min_award_quantity: Option<&str>,
    ) -> AtlasResult<SourcingEventLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sourcing_event_lines
                (organization_id, event_id, line_number, description, item_number, category,
                 quantity, uom, target_price, target_total, need_by_date, ship_to,
                 specifications, allow_partial_quantity, min_award_quantity)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8,
                    $9::numeric, $10::numeric, $11, $12, $13, $14, $15::numeric)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(event_id).bind(line_number).bind(description)
        .bind(item_number).bind(category)
        .bind(quantity).bind(uom)
        .bind(target_price).bind(target_total).bind(need_by_date).bind(ship_to)
        .bind(specifications).bind(allow_partial_quantity).bind(min_award_quantity)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_event_line(&row))
    }

    async fn list_event_lines(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingEventLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sourcing_event_lines WHERE event_id = $1 ORDER BY line_number"
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_event_line(&r)).collect())
    }

    async fn update_event_line_award(&self, line_id: Uuid, supplier_id: Uuid, supplier_name: Option<&str>, awarded_price: &str, awarded_quantity: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.sourcing_event_lines
            SET status = 'awarded', awarded_supplier_id = $2, awarded_supplier_name = $3,
                awarded_price = $4::numeric, awarded_quantity = $5::numeric, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(line_id).bind(supplier_id).bind(supplier_name).bind(awarded_price).bind(awarded_quantity)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Invites
    // ========================================================================

    async fn create_invite(&self, org_id: Uuid, event_id: Uuid, supplier_id: Uuid, supplier_name: Option<&str>, supplier_email: Option<&str>) -> AtlasResult<SourcingInvite> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sourcing_invites
                (organization_id, event_id, supplier_id, supplier_name, supplier_email)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(event_id).bind(supplier_id).bind(supplier_name).bind(supplier_email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_invite(&row))
    }

    async fn get_invite(&self, event_id: Uuid, supplier_id: Uuid) -> AtlasResult<Option<SourcingInvite>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sourcing_invites WHERE event_id = $1 AND supplier_id = $2"
        )
        .bind(event_id).bind(supplier_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_invite(&r)))
    }

    async fn list_invites(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingInvite>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sourcing_invites WHERE event_id = $1 ORDER BY created_at"
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_invite(&r)).collect())
    }

    async fn update_invite_status(&self, id: Uuid, status: &str, viewed_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.sourcing_invites
            SET status = $2, has_responded = CASE WHEN $2 = 'responded' THEN true ELSE has_responded END,
                responded_at = CASE WHEN $2 = 'responded' THEN now() ELSE responded_at END,
                viewed_at = COALESCE($3, viewed_at),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(status).bind(viewed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Responses
    // ========================================================================

    async fn create_response(
        &self, org_id: Uuid, event_id: Uuid, response_number: &str,
        supplier_id: Uuid, supplier_name: Option<&str>,
        cover_letter: Option<&str>, valid_until: Option<chrono::NaiveDate>,
        payment_terms: Option<&str>, lead_time_days: Option<i32>,
        warranty_months: Option<i32>, created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierResponse> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.supplier_responses
                (organization_id, event_id, response_number, supplier_id, supplier_name,
                 status, cover_letter, valid_until, payment_terms,
                 lead_time_days, warranty_months, submitted_at, created_by)
            VALUES ($1, $2, $3, $4, $5, 'submitted', $6, $7, $8, $9, $10, now(), $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(event_id).bind(response_number).bind(supplier_id).bind(supplier_name)
        .bind(cover_letter).bind(valid_until).bind(payment_terms)
        .bind(lead_time_days).bind(warranty_months).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_response(&row))
    }

    async fn get_response(&self, id: Uuid) -> AtlasResult<Option<SupplierResponse>> {
        let row = sqlx::query("SELECT * FROM _atlas.supplier_responses WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_response(&r)))
    }

    async fn list_responses(&self, event_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SupplierResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.supplier_responses
            WHERE event_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at
            "#,
        )
        .bind(event_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_response(&r)).collect())
    }

    async fn update_response_total(&self, id: Uuid, total: &str) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.supplier_responses SET total_amount = $2::numeric, updated_at = now() WHERE id = $1")
            .bind(id).bind(total)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_response_status(&self, id: Uuid, status: &str) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.supplier_responses SET status = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(status)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_response_score_total(&self, id: Uuid, total_score: &str, evaluated_by: Option<Uuid>) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.supplier_responses SET total_score = $2::numeric, evaluated_by = $3, evaluated_at = now(), updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(total_score).bind(evaluated_by)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_response_rank(&self, id: Uuid, rank: i32) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.supplier_responses SET rank = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(rank)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Response Lines
    // ========================================================================

    async fn create_response_line(
        &self, org_id: Uuid, response_id: Uuid, event_line_id: Uuid,
        line_number: i32, unit_price: &str, quantity: &str, line_amount: &str,
        discount_percent: Option<&str>, effective_price: Option<&str>,
        promised_delivery_date: Option<chrono::NaiveDate>, lead_time_days: Option<i32>,
        supplier_notes: Option<&str>,
    ) -> AtlasResult<SupplierResponseLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.supplier_response_lines
                (organization_id, response_id, event_line_id, line_number,
                 unit_price, quantity, line_amount, discount_percent, effective_price,
                 promised_delivery_date, lead_time_days, supplier_notes)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric,
                    $8::numeric, $9::numeric, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(response_id).bind(event_line_id).bind(line_number)
        .bind(unit_price).bind(quantity).bind(line_amount)
        .bind(discount_percent).bind(effective_price)
        .bind(promised_delivery_date).bind(lead_time_days).bind(supplier_notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_response_line(&row))
    }

    async fn list_response_lines(&self, response_id: Uuid) -> AtlasResult<Vec<SupplierResponseLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.supplier_response_lines WHERE response_id = $1 ORDER BY line_number"
        )
        .bind(response_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_response_line(&r)).collect())
    }

    // ========================================================================
    // Scoring
    // ========================================================================

    async fn create_scoring_criterion(
        &self, org_id: Uuid, event_id: Uuid, name: &str,
        description: Option<&str>, weight: &str, max_score: &str,
        criterion_type: &str, display_order: i32, is_mandatory: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ScoringCriterion> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.scoring_criteria
                (organization_id, event_id, name, description, weight, max_score,
                 criterion_type, display_order, is_mandatory, created_by)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(event_id).bind(name).bind(description)
        .bind(weight).bind(max_score).bind(criterion_type)
        .bind(display_order).bind(is_mandatory).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_criterion(&row))
    }

    async fn get_scoring_criterion(&self, id: Uuid) -> AtlasResult<Option<ScoringCriterion>> {
        let row = sqlx::query("SELECT * FROM _atlas.scoring_criteria WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_criterion(&r)))
    }

    async fn list_scoring_criteria(&self, event_id: Uuid) -> AtlasResult<Vec<ScoringCriterion>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.scoring_criteria WHERE event_id = $1 ORDER BY display_order"
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_criterion(&r)).collect())
    }

    async fn upsert_response_score(
        &self, org_id: Uuid, response_id: Uuid, criterion_id: Uuid,
        score: &str, weighted_score: &str, notes: Option<&str>,
        scored_by: Option<Uuid>,
    ) -> AtlasResult<ResponseScore> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.response_scores
                (organization_id, response_id, criterion_id, score, weighted_score, notes, scored_by, scored_at)
            VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6, $7, now())
            ON CONFLICT (response_id, criterion_id) DO UPDATE
                SET score = $4::numeric, weighted_score = $5::numeric,
                    notes = $6, scored_by = $7, scored_at = now(), updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(response_id).bind(criterion_id)
        .bind(score).bind(weighted_score).bind(notes).bind(scored_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_response_score(&row))
    }

    async fn list_response_scores(&self, response_id: Uuid) -> AtlasResult<Vec<ResponseScore>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.response_scores WHERE response_id = $1 ORDER BY created_at"
        )
        .bind(response_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_response_score(&r)).collect())
    }

    // ========================================================================
    // Awards
    // ========================================================================

    async fn create_award(
        &self, org_id: Uuid, event_id: Uuid, award_number: &str,
        award_method: &str, total_awarded_amount: &str,
        award_rationale: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingAward> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sourcing_awards
                (organization_id, event_id, award_number, award_method,
                 status, total_awarded_amount, award_rationale, created_by)
            VALUES ($1, $2, $3, $4, 'pending', $5::numeric, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(event_id).bind(award_number).bind(award_method)
        .bind(total_awarded_amount).bind(award_rationale).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_award(&row))
    }

    async fn get_award(&self, id: Uuid) -> AtlasResult<Option<SourcingAward>> {
        let row = sqlx::query("SELECT * FROM _atlas.sourcing_awards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_award(&r)))
    }

    async fn list_awards(&self, event_id: Uuid) -> AtlasResult<Vec<SourcingAward>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sourcing_awards WHERE event_id = $1 ORDER BY created_at DESC"
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_award(&r)).collect())
    }

    async fn update_award_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, rejected_reason: Option<&str>) -> AtlasResult<SourcingAward> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.sourcing_awards
            SET status = $2,
                approved_by = CASE WHEN $2 = 'approved' THEN $3 ELSE approved_by END,
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                rejected_reason = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_award(&row))
    }

    async fn create_award_line(
        &self, org_id: Uuid, award_id: Uuid, event_line_id: Uuid,
        response_id: Uuid, supplier_id: Uuid, supplier_name: Option<&str>,
        awarded_quantity: &str, awarded_unit_price: &str, awarded_amount: &str,
    ) -> AtlasResult<SourcingAwardLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sourcing_award_lines
                (organization_id, award_id, event_line_id, response_id,
                 supplier_id, supplier_name, awarded_quantity, awarded_unit_price, awarded_amount)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9::numeric)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(award_id).bind(event_line_id).bind(response_id)
        .bind(supplier_id).bind(supplier_name)
        .bind(awarded_quantity).bind(awarded_unit_price).bind(awarded_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_award_line(&row))
    }

    async fn list_award_lines(&self, award_id: Uuid) -> AtlasResult<Vec<SourcingAwardLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sourcing_award_lines WHERE award_id = $1 ORDER BY created_at"
        )
        .bind(award_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_award_line(&r)).collect())
    }

    // ========================================================================
    // Templates
    // ========================================================================

    async fn create_template(
        &self, org_id: Uuid, code: &str, name: &str,
        description: Option<&str>, default_event_type: &str,
        default_style: &str, default_scoring_method: &str,
        default_response_deadline_days: i32, currency_code: &str,
        default_bids_visible: bool, default_terms: Option<&str>,
        default_scoring_criteria: serde_json::Value,
        default_lines: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SourcingTemplate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sourcing_templates
                (organization_id, code, name, description, default_event_type, default_style,
                 default_scoring_method, default_response_deadline_days, currency_code,
                 default_bids_visible, default_terms, default_scoring_criteria,
                 default_lines, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, default_event_type = $5, default_style = $6,
                    default_scoring_method = $7, default_response_deadline_days = $8,
                    currency_code = $9, default_bids_visible = $10, default_terms = $11,
                    default_scoring_criteria = $12, default_lines = $13, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(default_event_type).bind(default_style).bind(default_scoring_method)
        .bind(default_response_deadline_days).bind(currency_code)
        .bind(default_bids_visible).bind(default_terms)
        .bind(default_scoring_criteria).bind(default_lines).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SourcingTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sourcing_templates WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<SourcingTemplate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sourcing_templates WHERE organization_id = $1 AND is_active = true ORDER BY name"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_template(&r)).collect())
    }

    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.sourcing_templates SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
