//! Collections Repository
//!
//! PostgreSQL storage for credit profiles, collection strategies, cases,
//! interactions, promises to pay, dunning campaigns, dunning letters,
//! aging snapshots, and write-off requests.

use atlas_shared::{
    CustomerCreditProfile, CollectionStrategy, CollectionCase,
    CustomerInteraction, PromiseToPay, DunningCampaign, DunningLetter,
    ReceivablesAgingSnapshot, WriteOffRequest,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for collections & credit management data storage
#[async_trait]
pub trait CollectionsRepository: Send + Sync {
    // Credit Profiles
    async fn create_credit_profile(
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
    ) -> AtlasResult<CustomerCreditProfile>;

    async fn get_credit_profile(&self, org_id: Uuid, customer_id: Uuid) -> AtlasResult<Option<CustomerCreditProfile>>;
    async fn get_credit_profile_by_id(&self, id: Uuid) -> AtlasResult<Option<CustomerCreditProfile>>;
    async fn list_credit_profiles(&self, org_id: Uuid, status: Option<&str>, risk_classification: Option<&str>) -> AtlasResult<Vec<CustomerCreditProfile>>;
    async fn update_credit_profile(
        &self,
        id: Uuid,
        credit_limit: Option<&str>,
        credit_used: Option<&str>,
        risk_classification: Option<&str>,
        credit_score: Option<i32>,
        payment_terms: Option<&str>,
        average_days_to_pay: Option<&str>,
        overdue_invoice_count: Option<i32>,
        total_overdue_amount: Option<&str>,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        credit_hold: Option<bool>,
        credit_hold_reason: Option<&str>,
        credit_hold_date: Option<chrono::DateTime<chrono::Utc>>,
        credit_hold_by: Option<Uuid>,
        last_review_date: Option<chrono::NaiveDate>,
        next_review_date: Option<chrono::NaiveDate>,
        status: Option<&str>,
    ) -> AtlasResult<CustomerCreditProfile>;

    // Collection Strategies
    async fn create_strategy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        strategy_type: &str,
        applicable_risk_classifications: serde_json::Value,
        trigger_aging_buckets: serde_json::Value,
        overdue_amount_threshold: &str,
        actions: serde_json::Value,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CollectionStrategy>;

    async fn get_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CollectionStrategy>>;
    async fn list_strategies(&self, org_id: Uuid) -> AtlasResult<Vec<CollectionStrategy>>;
    async fn delete_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Collection Cases
    async fn create_case(
        &self,
        org_id: Uuid,
        case_number: &str,
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
    ) -> AtlasResult<CollectionCase>;

    async fn get_case(&self, id: Uuid) -> AtlasResult<Option<CollectionCase>>;
    async fn get_case_by_number(&self, org_id: Uuid, case_number: &str) -> AtlasResult<Option<CollectionCase>>;
    async fn list_cases(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>, assigned_to: Option<Uuid>) -> AtlasResult<Vec<CollectionCase>>;
    async fn update_case_status(
        &self,
        id: Uuid,
        status: &str,
        current_step: Option<i32>,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        last_action_date: Option<chrono::NaiveDate>,
        next_action_date: Option<chrono::NaiveDate>,
        resolution_type: Option<&str>,
        resolution_notes: Option<&str>,
        resolved_date: Option<chrono::NaiveDate>,
        closed_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<CollectionCase>;

    // Customer Interactions
    async fn create_interaction(
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
    ) -> AtlasResult<CustomerInteraction>;

    async fn get_interaction(&self, id: Uuid) -> AtlasResult<Option<CustomerInteraction>>;
    async fn list_interactions(&self, org_id: Uuid, case_id: Option<Uuid>, customer_id: Option<Uuid>) -> AtlasResult<Vec<CustomerInteraction>>;

    // Promises to Pay
    async fn create_promise_to_pay(
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
    ) -> AtlasResult<PromiseToPay>;

    async fn get_promise_to_pay(&self, id: Uuid) -> AtlasResult<Option<PromiseToPay>>;
    async fn list_promises_to_pay(&self, org_id: Uuid, customer_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<PromiseToPay>>;
    async fn update_promise_status(
        &self,
        id: Uuid,
        status: &str,
        paid_amount: Option<&str>,
        remaining_amount: Option<&str>,
        broken_date: Option<chrono::NaiveDate>,
        broken_reason: Option<&str>,
    ) -> AtlasResult<PromiseToPay>;

    // Dunning Campaigns
    async fn create_dunning_campaign(
        &self,
        org_id: Uuid,
        campaign_number: &str,
        name: &str,
        description: Option<&str>,
        dunning_level: &str,
        communication_method: &str,
        template_id: Option<Uuid>,
        template_name: Option<&str>,
        min_overdue_days: i32,
        min_overdue_amount: &str,
        target_risk_classifications: serde_json::Value,
        exclude_active_cases: bool,
        scheduled_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningCampaign>;

    async fn get_dunning_campaign(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<Option<DunningCampaign>>;
    async fn list_dunning_campaigns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DunningCampaign>>;
    async fn update_dunning_campaign_status(&self, id: Uuid, status: &str, sent_date: Option<chrono::NaiveDate>) -> AtlasResult<DunningCampaign>;

    // Dunning Letters
    async fn create_dunning_letter(
        &self,
        org_id: Uuid,
        campaign_id: Option<Uuid>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        dunning_level: &str,
        communication_method: &str,
        total_overdue_amount: &str,
        overdue_invoice_count: i32,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        invoice_details: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetter>;

    async fn list_dunning_letters(&self, org_id: Uuid, campaign_id: Option<Uuid>, customer_id: Option<Uuid>) -> AtlasResult<Vec<DunningLetter>>;
    async fn update_dunning_letter_status(&self, id: Uuid, status: &str) -> AtlasResult<DunningLetter>;

    // Aging Snapshots
    async fn create_aging_snapshot(
        &self,
        org_id: Uuid,
        snapshot_date: chrono::NaiveDate,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        total_outstanding: &str,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        count_current: i32,
        count_1_30: i32,
        count_31_60: i32,
        count_61_90: i32,
        count_91_120: i32,
        count_121_plus: i32,
        weighted_average_days_overdue: Option<&str>,
        overdue_percent: Option<&str>,
    ) -> AtlasResult<ReceivablesAgingSnapshot>;

    async fn list_aging_snapshots(&self, org_id: Uuid, snapshot_date: chrono::NaiveDate) -> AtlasResult<Vec<ReceivablesAgingSnapshot>>;

    // Write-Off Requests
    async fn create_write_off_request(
        &self,
        org_id: Uuid,
        request_number: &str,
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
    ) -> AtlasResult<WriteOffRequest>;

    async fn get_write_off_request(&self, id: Uuid) -> AtlasResult<Option<WriteOffRequest>>;
    async fn list_write_off_requests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WriteOffRequest>>;
    async fn update_write_off_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
        journal_entry_id: Option<Uuid>,
    ) -> AtlasResult<WriteOffRequest>;
}

/// PostgreSQL implementation
pub struct PostgresCollectionsRepository {
    pool: PgPool,
}

impl PostgresCollectionsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_credit_profile(&self, row: &sqlx::postgres::PgRow) -> CustomerCreditProfile {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        CustomerCreditProfile {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            customer_id: row.get("customer_id"),
            customer_number: row.get("customer_number"),
            customer_name: row.get("customer_name"),
            credit_limit: get_num(row, "credit_limit"),
            credit_used: get_num(row, "credit_used"),
            credit_available: get_num(row, "credit_available"),
            risk_classification: row.get("risk_classification"),
            credit_score: row.get("credit_score"),
            external_credit_rating: row.get("external_credit_rating"),
            external_rating_agency: row.get("external_rating_agency"),
            external_rating_date: row.get("external_rating_date"),
            payment_terms: row.get("payment_terms"),
            average_days_to_pay: row.try_get("average_days_to_pay").unwrap_or(None),
            overdue_invoice_count: row.get("overdue_invoice_count"),
            total_overdue_amount: get_num(row, "total_overdue_amount"),
            oldest_overdue_date: row.get("oldest_overdue_date"),
            credit_hold: row.get("credit_hold"),
            credit_hold_reason: row.get("credit_hold_reason"),
            credit_hold_date: row.get("credit_hold_date"),
            credit_hold_by: row.get("credit_hold_by"),
            last_review_date: row.get("last_review_date"),
            next_review_date: row.get("next_review_date"),
            status: row.get("status"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl CollectionsRepository for PostgresCollectionsRepository {
    // ========================================================================
    // Credit Profiles
    // ========================================================================

    async fn create_credit_profile(
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.customer_credit_profiles
                (organization_id, customer_id, customer_number, customer_name,
                 credit_limit, risk_classification, credit_score,
                 external_credit_rating, external_rating_agency, external_rating_date,
                 payment_terms, next_review_date, created_by)
            VALUES ($1, $2, $3, $4, $5::numeric, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (organization_id, customer_id) DO UPDATE
                SET customer_number = $3, customer_name = $4,
                    credit_limit = $5::numeric, credit_available = $5::numeric - credit_used,
                    risk_classification = $6, credit_score = $7,
                    external_credit_rating = $8, external_rating_agency = $9,
                    external_rating_date = $10, payment_terms = $11,
                    next_review_date = $12, status = 'active', updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(credit_limit).bind(risk_classification).bind(credit_score)
        .bind(external_credit_rating).bind(external_rating_agency).bind(external_rating_date)
        .bind(payment_terms).bind(next_review_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_credit_profile(&row))
    }

    async fn get_credit_profile(&self, org_id: Uuid, customer_id: Uuid) -> AtlasResult<Option<CustomerCreditProfile>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.customer_credit_profiles WHERE organization_id = $1 AND customer_id = $2 AND status != 'inactive'"
        )
        .bind(org_id).bind(customer_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_credit_profile(&r)))
    }

    async fn get_credit_profile_by_id(&self, id: Uuid) -> AtlasResult<Option<CustomerCreditProfile>> {
        let row = sqlx::query("SELECT * FROM _atlas.customer_credit_profiles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_credit_profile(&r)))
    }

    async fn list_credit_profiles(&self, org_id: Uuid, status: Option<&str>, risk_classification: Option<&str>) -> AtlasResult<Vec<CustomerCreditProfile>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.customer_credit_profiles
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR risk_classification = $3)
            ORDER BY customer_name
            "#,
        )
        .bind(org_id).bind(status).bind(risk_classification)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_credit_profile(r)).collect())
    }

    async fn update_credit_profile(
        &self,
        id: Uuid,
        credit_limit: Option<&str>,
        credit_used: Option<&str>,
        risk_classification: Option<&str>,
        credit_score: Option<i32>,
        payment_terms: Option<&str>,
        average_days_to_pay: Option<&str>,
        overdue_invoice_count: Option<i32>,
        total_overdue_amount: Option<&str>,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        credit_hold: Option<bool>,
        credit_hold_reason: Option<&str>,
        credit_hold_date: Option<chrono::DateTime<chrono::Utc>>,
        credit_hold_by: Option<Uuid>,
        last_review_date: Option<chrono::NaiveDate>,
        next_review_date: Option<chrono::NaiveDate>,
        status: Option<&str>,
    ) -> AtlasResult<CustomerCreditProfile> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.customer_credit_profiles
            SET credit_limit = COALESCE($2::numeric, credit_limit),
                credit_used = COALESCE($3::numeric, credit_used),
                credit_available = CASE WHEN $2 IS NOT NULL THEN $2::numeric - COALESCE(credit_used, 0)
                                        WHEN $3 IS NOT NULL THEN COALESCE(credit_limit, 0) - $3::numeric
                                        ELSE credit_available END,
                risk_classification = COALESCE($4, risk_classification),
                credit_score = COALESCE($5, credit_score),
                payment_terms = COALESCE($6, payment_terms),
                average_days_to_pay = COALESCE($7::numeric, average_days_to_pay),
                overdue_invoice_count = COALESCE($8, overdue_invoice_count),
                total_overdue_amount = COALESCE($9::numeric, total_overdue_amount),
                oldest_overdue_date = COALESCE($10, oldest_overdue_date),
                credit_hold = COALESCE($11, credit_hold),
                credit_hold_reason = $12,
                credit_hold_date = $13,
                credit_hold_by = $14,
                last_review_date = COALESCE($15, last_review_date),
                next_review_date = $16,
                status = COALESCE($17, status),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(credit_limit).bind(credit_used)
        .bind(risk_classification).bind(credit_score)
        .bind(payment_terms).bind(average_days_to_pay)
        .bind(overdue_invoice_count).bind(total_overdue_amount)
        .bind(oldest_overdue_date)
        .bind(credit_hold).bind(credit_hold_reason).bind(credit_hold_date).bind(credit_hold_by)
        .bind(last_review_date).bind(next_review_date).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_credit_profile(&row))
    }

    // ========================================================================
    // Collection Strategies
    // ========================================================================

    async fn create_strategy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        strategy_type: &str,
        applicable_risk_classifications: serde_json::Value,
        trigger_aging_buckets: serde_json::Value,
        overdue_amount_threshold: &str,
        actions: serde_json::Value,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CollectionStrategy> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.collection_strategies
                (organization_id, code, name, description, strategy_type,
                 applicable_risk_classifications, trigger_aging_buckets,
                 overdue_amount_threshold, actions, priority, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9, $10, $11)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, strategy_type = $5,
                    applicable_risk_classifications = $6, trigger_aging_buckets = $7,
                    overdue_amount_threshold = $8::numeric, actions = $9,
                    priority = $10, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(strategy_type)
        .bind(applicable_risk_classifications).bind(trigger_aging_buckets)
        .bind(overdue_amount_threshold).bind(actions).bind(priority).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(CollectionStrategy {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            strategy_type: row.get("strategy_type"),
            applicable_risk_classifications: row.try_get("applicable_risk_classifications").unwrap_or(serde_json::json!([])),
            trigger_aging_buckets: row.try_get("trigger_aging_buckets").unwrap_or(serde_json::json!([])),
            overdue_amount_threshold: row.try_get("overdue_amount_threshold").map(|v: serde_json::Value| v.to_string()).unwrap_or("0".to_string()),
            actions: row.try_get("actions").unwrap_or(serde_json::json!([])),
            priority: row.get("priority"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CollectionStrategy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.collection_strategies WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| CollectionStrategy {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            strategy_type: r.get("strategy_type"),
            applicable_risk_classifications: r.try_get("applicable_risk_classifications").unwrap_or(serde_json::json!([])),
            trigger_aging_buckets: r.try_get("trigger_aging_buckets").unwrap_or(serde_json::json!([])),
            overdue_amount_threshold: r.try_get("overdue_amount_threshold").map(|v: serde_json::Value| v.to_string()).unwrap_or("0".to_string()),
            actions: r.try_get("actions").unwrap_or(serde_json::json!([])),
            priority: r.get("priority"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn list_strategies(&self, org_id: Uuid) -> AtlasResult<Vec<CollectionStrategy>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.collection_strategies WHERE organization_id = $1 AND is_active = true ORDER BY priority DESC, code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| CollectionStrategy {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            strategy_type: r.get("strategy_type"),
            applicable_risk_classifications: r.try_get("applicable_risk_classifications").unwrap_or(serde_json::json!([])),
            trigger_aging_buckets: r.try_get("trigger_aging_buckets").unwrap_or(serde_json::json!([])),
            overdue_amount_threshold: r.try_get("overdue_amount_threshold").map(|v: serde_json::Value| v.to_string()).unwrap_or("0".to_string()),
            actions: r.try_get("actions").unwrap_or(serde_json::json!([])),
            priority: r.get("priority"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.collection_strategies SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Collection Cases
    // ========================================================================

    async fn create_case(
        &self,
        org_id: Uuid,
        case_number: &str,
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.collection_cases
                (organization_id, case_number, customer_id, customer_number, customer_name,
                 strategy_id, assigned_to, assigned_to_name, case_type, priority,
                 total_overdue_amount, total_disputed_amount, total_invoiced_amount,
                 overdue_invoice_count, oldest_overdue_date, related_invoice_ids, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::numeric, $12::numeric, $13::numeric, $14, $15, $16, $17)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(case_number).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(strategy_id).bind(assigned_to).bind(assigned_to_name).bind(case_type).bind(priority)
        .bind(total_overdue_amount).bind(total_disputed_amount).bind(total_invoiced_amount)
        .bind(overdue_invoice_count).bind(oldest_overdue_date).bind(related_invoice_ids).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_case(&row))
    }

    async fn get_case(&self, id: Uuid) -> AtlasResult<Option<CollectionCase>> {
        let row = sqlx::query("SELECT * FROM _atlas.collection_cases WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_case(&r)))
    }

    async fn get_case_by_number(&self, org_id: Uuid, case_number: &str) -> AtlasResult<Option<CollectionCase>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.collection_cases WHERE organization_id = $1 AND case_number = $2"
        )
        .bind(org_id).bind(case_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_case(&r)))
    }

    async fn list_cases(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>, assigned_to: Option<Uuid>) -> AtlasResult<Vec<CollectionCase>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.collection_cases
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
              AND ($4::uuid IS NULL OR assigned_to = $4)
            ORDER BY
                CASE priority
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                END,
                opened_date DESC
            "#,
        )
        .bind(org_id).bind(status).bind(customer_id).bind(assigned_to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_case).collect())
    }

    async fn update_case_status(
        &self,
        id: Uuid,
        status: &str,
        current_step: Option<i32>,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        last_action_date: Option<chrono::NaiveDate>,
        next_action_date: Option<chrono::NaiveDate>,
        resolution_type: Option<&str>,
        resolution_notes: Option<&str>,
        resolved_date: Option<chrono::NaiveDate>,
        closed_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<CollectionCase> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.collection_cases
            SET status = $2,
                current_step = COALESCE($3, current_step),
                assigned_to = COALESCE($4, assigned_to),
                assigned_to_name = $5,
                last_action_date = COALESCE($6, last_action_date),
                next_action_date = $7,
                resolution_type = $8,
                resolution_notes = $9,
                resolved_date = COALESCE($10, resolved_date),
                closed_date = COALESCE($11, closed_date),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(current_step)
        .bind(assigned_to).bind(assigned_to_name)
        .bind(last_action_date).bind(next_action_date)
        .bind(resolution_type).bind(resolution_notes)
        .bind(resolved_date).bind(closed_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_case(&row))
    }

    // ========================================================================
    // Customer Interactions
    // ========================================================================

    async fn create_interaction(
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.customer_interactions
                (organization_id, case_id, customer_id, customer_number, customer_name,
                 interaction_type, direction, contact_name, contact_role,
                 contact_phone, contact_email, subject, body, outcome,
                 follow_up_date, follow_up_notes, performed_by, performed_by_name, duration_minutes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(case_id).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(interaction_type).bind(direction)
        .bind(contact_name).bind(contact_role).bind(contact_phone).bind(contact_email)
        .bind(subject).bind(body).bind(outcome)
        .bind(follow_up_date).bind(follow_up_notes)
        .bind(performed_by).bind(performed_by_name).bind(duration_minutes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_interaction(&row))
    }

    async fn get_interaction(&self, id: Uuid) -> AtlasResult<Option<CustomerInteraction>> {
        let row = sqlx::query("SELECT * FROM _atlas.customer_interactions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_interaction(&r)))
    }

    async fn list_interactions(&self, org_id: Uuid, case_id: Option<Uuid>, customer_id: Option<Uuid>) -> AtlasResult<Vec<CustomerInteraction>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.customer_interactions
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR case_id = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
            ORDER BY performed_at DESC
            "#,
        )
        .bind(org_id).bind(case_id).bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_interaction).collect())
    }

    // ========================================================================
    // Promises to Pay
    // ========================================================================

    async fn create_promise_to_pay(
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.promise_to_pay
                (organization_id, case_id, customer_id, customer_number, customer_name,
                 promise_type, promised_amount, remaining_amount, promise_date,
                 installment_count, installment_frequency,
                 related_invoice_ids, promised_by_name, promised_by_role, notes, recorded_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $7::numeric, $8,
                    $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(case_id).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(promise_type).bind(promised_amount).bind(promise_date)
        .bind(installment_count).bind(installment_frequency)
        .bind(related_invoice_ids).bind(promised_by_name).bind(promised_by_role)
        .bind(notes).bind(recorded_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_ptp(&row))
    }

    async fn get_promise_to_pay(&self, id: Uuid) -> AtlasResult<Option<PromiseToPay>> {
        let row = sqlx::query("SELECT * FROM _atlas.promise_to_pay WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_ptp(&r)))
    }

    async fn list_promises_to_pay(&self, org_id: Uuid, customer_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<PromiseToPay>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.promise_to_pay
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY promise_date
            "#,
        )
        .bind(org_id).bind(customer_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_ptp).collect())
    }

    async fn update_promise_status(
        &self,
        id: Uuid,
        status: &str,
        paid_amount: Option<&str>,
        remaining_amount: Option<&str>,
        broken_date: Option<chrono::NaiveDate>,
        broken_reason: Option<&str>,
    ) -> AtlasResult<PromiseToPay> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.promise_to_pay
            SET status = $2,
                paid_amount = COALESCE($3::numeric, paid_amount),
                remaining_amount = COALESCE($4::numeric, remaining_amount),
                broken_date = COALESCE($5, broken_date),
                broken_reason = $6,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(paid_amount).bind(remaining_amount)
        .bind(broken_date).bind(broken_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_ptp(&row))
    }

    // ========================================================================
    // Dunning Campaigns
    // ========================================================================

    async fn create_dunning_campaign(
        &self,
        org_id: Uuid,
        campaign_number: &str,
        name: &str,
        description: Option<&str>,
        dunning_level: &str,
        communication_method: &str,
        template_id: Option<Uuid>,
        template_name: Option<&str>,
        min_overdue_days: i32,
        min_overdue_amount: &str,
        target_risk_classifications: serde_json::Value,
        exclude_active_cases: bool,
        scheduled_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningCampaign> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.dunning_campaigns
                (organization_id, campaign_number, name, description,
                 dunning_level, communication_method, template_id, template_name,
                 min_overdue_days, min_overdue_amount,
                 target_risk_classifications, exclude_active_cases,
                 scheduled_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::numeric, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(campaign_number).bind(name).bind(description)
        .bind(dunning_level).bind(communication_method).bind(template_id).bind(template_name)
        .bind(min_overdue_days).bind(min_overdue_amount)
        .bind(target_risk_classifications).bind(exclude_active_cases)
        .bind(scheduled_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_campaign(&row))
    }

    async fn get_dunning_campaign(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<Option<DunningCampaign>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dunning_campaigns WHERE organization_id = $1 AND campaign_number = $2"
        )
        .bind(org_id).bind(campaign_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_campaign(&r)))
    }

    async fn list_dunning_campaigns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DunningCampaign>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.dunning_campaigns
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_campaign).collect())
    }

    async fn update_dunning_campaign_status(&self, id: Uuid, status: &str, sent_date: Option<chrono::NaiveDate>) -> AtlasResult<DunningCampaign> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.dunning_campaigns
            SET status = $2, sent_date = COALESCE($3, sent_date), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(sent_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign(&row))
    }

    // ========================================================================
    // Dunning Letters
    // ========================================================================

    async fn create_dunning_letter(
        &self,
        org_id: Uuid,
        campaign_id: Option<Uuid>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        dunning_level: &str,
        communication_method: &str,
        total_overdue_amount: &str,
        overdue_invoice_count: i32,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        invoice_details: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetter> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.dunning_letters
                (organization_id, campaign_id, customer_id, customer_number, customer_name,
                 dunning_level, communication_method,
                 total_overdue_amount, overdue_invoice_count, oldest_overdue_date,
                 aging_current, aging_1_30, aging_31_60, aging_61_90, aging_91_120, aging_121_plus,
                 invoice_details, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9, $10,
                    $11::numeric, $12::numeric, $13::numeric, $14::numeric, $15::numeric, $16::numeric,
                    $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(campaign_id).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(dunning_level).bind(communication_method)
        .bind(total_overdue_amount).bind(overdue_invoice_count).bind(oldest_overdue_date)
        .bind(aging_current).bind(aging_1_30).bind(aging_31_60).bind(aging_61_90).bind(aging_91_120).bind(aging_121_plus)
        .bind(invoice_details).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_letter(&row))
    }

    async fn list_dunning_letters(&self, org_id: Uuid, campaign_id: Option<Uuid>, customer_id: Option<Uuid>) -> AtlasResult<Vec<DunningLetter>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.dunning_letters
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR campaign_id = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(campaign_id).bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_letter).collect())
    }

    async fn update_dunning_letter_status(&self, id: Uuid, status: &str) -> AtlasResult<DunningLetter> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.dunning_letters
            SET status = $2,
                sent_at = CASE WHEN $2 = 'sent' AND sent_at IS NULL THEN now() ELSE sent_at END,
                delivered_at = CASE WHEN $2 = 'delivered' AND delivered_at IS NULL THEN now() ELSE delivered_at END,
                viewed_at = CASE WHEN $2 = 'viewed' AND viewed_at IS NULL THEN now() ELSE viewed_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_letter(&row))
    }

    // ========================================================================
    // Aging Snapshots
    // ========================================================================

    async fn create_aging_snapshot(
        &self,
        org_id: Uuid,
        snapshot_date: chrono::NaiveDate,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        total_outstanding: &str,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        count_current: i32,
        count_1_30: i32,
        count_31_60: i32,
        count_61_90: i32,
        count_91_120: i32,
        count_121_plus: i32,
        weighted_average_days_overdue: Option<&str>,
        overdue_percent: Option<&str>,
    ) -> AtlasResult<ReceivablesAgingSnapshot> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receivables_aging_snapshots
                (organization_id, snapshot_date, customer_id, customer_number, customer_name,
                 total_outstanding, aging_current, aging_1_30, aging_31_60, aging_61_90,
                 aging_91_120, aging_121_plus, count_current, count_1_30, count_31_60,
                 count_61_90, count_91_120, count_121_plus,
                 weighted_average_days_overdue, overdue_percent)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8::numeric, $9::numeric,
                    $10::numeric, $11::numeric, $12::numeric, $13, $14, $15, $16, $17, $18,
                    $19::numeric, $20::numeric)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(snapshot_date).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(total_outstanding).bind(aging_current).bind(aging_1_30).bind(aging_31_60).bind(aging_61_90)
        .bind(aging_91_120).bind(aging_121_plus).bind(count_current).bind(count_1_30).bind(count_31_60)
        .bind(count_61_90).bind(count_91_120).bind(count_121_plus)
        .bind(weighted_average_days_overdue).bind(overdue_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_snapshot(&row))
    }

    async fn list_aging_snapshots(&self, org_id: Uuid, snapshot_date: chrono::NaiveDate) -> AtlasResult<Vec<ReceivablesAgingSnapshot>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.receivables_aging_snapshots WHERE organization_id = $1 AND snapshot_date = $2 ORDER BY customer_name"
        )
        .bind(org_id).bind(snapshot_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_snapshot).collect())
    }

    // ========================================================================
    // Write-Off Requests
    // ========================================================================

    async fn create_write_off_request(
        &self,
        org_id: Uuid,
        request_number: &str,
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.write_off_requests
                (organization_id, request_number, customer_id, customer_number, customer_name,
                 write_off_type, write_off_amount, write_off_account_code,
                 reason, related_invoice_ids, case_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(request_number).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(write_off_type).bind(write_off_amount).bind(write_off_account_code)
        .bind(reason).bind(related_invoice_ids).bind(case_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_write_off(&row))
    }

    async fn get_write_off_request(&self, id: Uuid) -> AtlasResult<Option<WriteOffRequest>> {
        let row = sqlx::query("SELECT * FROM _atlas.write_off_requests WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_write_off(&r)))
    }

    async fn list_write_off_requests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WriteOffRequest>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.write_off_requests
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_write_off).collect())
    }

    async fn update_write_off_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
        journal_entry_id: Option<Uuid>,
    ) -> AtlasResult<WriteOffRequest> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.write_off_requests
            SET status = $2,
                submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $3 IS NOT NULL AND submitted_at IS NULL THEN now() ELSE submitted_at END,
                approved_by = COALESCE($4, approved_by),
                approved_at = CASE WHEN $4 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                rejected_reason = $5,
                journal_entry_id = COALESCE($6, journal_entry_id),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(submitted_by).bind(approved_by)
        .bind(rejected_reason).bind(journal_entry_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_write_off(&row))
    }
}

// ============================================================================
// Row-to-struct helper functions
// ============================================================================

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_case(row: &sqlx::postgres::PgRow) -> CollectionCase {
    CollectionCase {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        case_number: row.get("case_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        strategy_id: row.get("strategy_id"),
        assigned_to: row.get("assigned_to"),
        assigned_to_name: row.get("assigned_to_name"),
        case_type: row.get("case_type"),
        status: row.get("status"),
        priority: row.get("priority"),
        total_overdue_amount: get_num(row, "total_overdue_amount"),
        total_disputed_amount: get_num(row, "total_disputed_amount"),
        total_invoiced_amount: get_num(row, "total_invoiced_amount"),
        overdue_invoice_count: row.get("overdue_invoice_count"),
        oldest_overdue_date: row.get("oldest_overdue_date"),
        current_step: row.get("current_step"),
        opened_date: row.get("opened_date"),
        target_resolution_date: row.get("target_resolution_date"),
        resolved_date: row.get("resolved_date"),
        closed_date: row.get("closed_date"),
        last_action_date: row.get("last_action_date"),
        next_action_date: row.get("next_action_date"),
        resolution_type: row.get("resolution_type"),
        resolution_notes: row.get("resolution_notes"),
        related_invoice_ids: row.try_get("related_invoice_ids").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_interaction(row: &sqlx::postgres::PgRow) -> CustomerInteraction {
    CustomerInteraction {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        case_id: row.get("case_id"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        interaction_type: row.get("interaction_type"),
        direction: row.get("direction"),
        contact_name: row.get("contact_name"),
        contact_role: row.get("contact_role"),
        contact_phone: row.get("contact_phone"),
        contact_email: row.get("contact_email"),
        subject: row.get("subject"),
        body: row.get("body"),
        outcome: row.get("outcome"),
        follow_up_date: row.get("follow_up_date"),
        follow_up_notes: row.get("follow_up_notes"),
        performed_by: row.get("performed_by"),
        performed_by_name: row.get("performed_by_name"),
        performed_at: row.get("performed_at"),
        duration_minutes: row.get("duration_minutes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_ptp(row: &sqlx::postgres::PgRow) -> PromiseToPay {
    PromiseToPay {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        case_id: row.get("case_id"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        promise_type: row.get("promise_type"),
        promised_amount: get_num(row, "promised_amount"),
        paid_amount: get_num(row, "paid_amount"),
        remaining_amount: get_num(row, "remaining_amount"),
        promise_date: row.get("promise_date"),
        installment_count: row.get("installment_count"),
        installment_frequency: row.get("installment_frequency"),
        status: row.get("status"),
        broken_date: row.get("broken_date"),
        broken_reason: row.get("broken_reason"),
        related_invoice_ids: row.try_get("related_invoice_ids").unwrap_or(serde_json::json!([])),
        promised_by_name: row.get("promised_by_name"),
        promised_by_role: row.get("promised_by_role"),
        notes: row.get("notes"),
        recorded_by: row.get("recorded_by"),
        recorded_at: row.get("recorded_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_campaign(row: &sqlx::postgres::PgRow) -> DunningCampaign {
    DunningCampaign {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        campaign_number: row.get("campaign_number"),
        name: row.get("name"),
        description: row.get("description"),
        dunning_level: row.get("dunning_level"),
        communication_method: row.get("communication_method"),
        template_id: row.get("template_id"),
        template_name: row.get("template_name"),
        min_overdue_days: row.get("min_overdue_days"),
        min_overdue_amount: get_num(row, "min_overdue_amount"),
        target_risk_classifications: row.try_get("target_risk_classifications").unwrap_or(serde_json::json!([])),
        exclude_active_cases: row.get("exclude_active_cases"),
        scheduled_date: row.get("scheduled_date"),
        sent_date: row.get("sent_date"),
        target_customer_count: row.get("target_customer_count"),
        sent_count: row.get("sent_count"),
        failed_count: row.get("failed_count"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_letter(row: &sqlx::postgres::PgRow) -> DunningLetter {
    DunningLetter {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        campaign_id: row.get("campaign_id"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        customer_address: row.try_get("customer_address").unwrap_or(None),
        customer_email: row.get("customer_email"),
        dunning_level: row.get("dunning_level"),
        communication_method: row.get("communication_method"),
        total_overdue_amount: get_num(row, "total_overdue_amount"),
        overdue_invoice_count: row.get("overdue_invoice_count"),
        oldest_overdue_date: row.get("oldest_overdue_date"),
        aging_current: get_num(row, "aging_current"),
        aging_1_30: get_num(row, "aging_1_30"),
        aging_31_60: get_num(row, "aging_31_60"),
        aging_61_90: get_num(row, "aging_61_90"),
        aging_91_120: get_num(row, "aging_91_120"),
        aging_121_plus: get_num(row, "aging_121_plus"),
        status: row.get("status"),
        sent_at: row.get("sent_at"),
        delivered_at: row.get("delivered_at"),
        viewed_at: row.get("viewed_at"),
        failure_reason: row.get("failure_reason"),
        invoice_details: row.try_get("invoice_details").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_snapshot(row: &sqlx::postgres::PgRow) -> ReceivablesAgingSnapshot {
    ReceivablesAgingSnapshot {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        snapshot_date: row.get("snapshot_date"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        total_outstanding: get_num(row, "total_outstanding"),
        aging_current: get_num(row, "aging_current"),
        aging_1_30: get_num(row, "aging_1_30"),
        aging_31_60: get_num(row, "aging_31_60"),
        aging_61_90: get_num(row, "aging_61_90"),
        aging_91_120: get_num(row, "aging_91_120"),
        aging_121_plus: get_num(row, "aging_121_plus"),
        count_current: row.get("count_current"),
        count_1_30: row.get("count_1_30"),
        count_31_60: row.get("count_31_60"),
        count_61_90: row.get("count_61_90"),
        count_91_120: row.get("count_91_120"),
        count_121_plus: row.get("count_121_plus"),
        weighted_average_days_overdue: row.try_get("weighted_average_days_overdue").unwrap_or(None).map(|v: serde_json::Value| v.to_string()),
        overdue_percent: row.try_get("overdue_percent").unwrap_or(None).map(|v: serde_json::Value| v.to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
    }
}

fn row_to_write_off(row: &sqlx::postgres::PgRow) -> WriteOffRequest {
    WriteOffRequest {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        request_number: row.get("request_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        write_off_type: row.get("write_off_type"),
        write_off_amount: get_num(row, "write_off_amount"),
        write_off_account_code: row.get("write_off_account_code"),
        reason: row.get("reason"),
        related_invoice_ids: row.try_get("related_invoice_ids").unwrap_or(serde_json::json!([])),
        case_id: row.get("case_id"),
        status: row.get("status"),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        rejected_reason: row.get("rejected_reason"),
        journal_entry_id: row.get("journal_entry_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
