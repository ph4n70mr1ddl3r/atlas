//! Credit Management Repository
//!
//! PostgreSQL storage for credit management data.

use atlas_shared::{
    CreditScoringModel, CreditProfile, CreditLimit, CreditCheckRule,
    CreditExposure, CreditHold, CreditReview, CreditManagementDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for Credit Management data storage
#[async_trait]
pub trait CreditManagementRepository: Send + Sync {
    // Scoring Models
    async fn create_scoring_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        model_type: &str,
        scoring_criteria: serde_json::Value,
        score_ranges: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditScoringModel>;

    async fn get_scoring_model(&self, id: Uuid) -> AtlasResult<Option<CreditScoringModel>>;
    async fn get_scoring_model_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CreditScoringModel>>;
    async fn list_scoring_models(&self, org_id: Uuid) -> AtlasResult<Vec<CreditScoringModel>>;
    async fn delete_scoring_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Credit Profiles
    async fn create_profile(
        &self,
        org_id: Uuid,
        profile_number: &str,
        profile_name: &str,
        description: Option<&str>,
        profile_type: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        customer_group_id: Option<Uuid>,
        customer_group_name: Option<&str>,
        scoring_model_id: Option<Uuid>,
        review_frequency_days: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditProfile>;

    async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<CreditProfile>>;
    async fn get_profile_by_number(&self, org_id: Uuid, profile_number: &str) -> AtlasResult<Option<CreditProfile>>;
    async fn get_profile_by_customer(&self, org_id: Uuid, customer_id: Uuid) -> AtlasResult<Option<CreditProfile>>;
    async fn list_profiles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CreditProfile>>;
    async fn update_profile_status(&self, id: Uuid, status: &str) -> AtlasResult<CreditProfile>;
    async fn update_profile_score(
        &self,
        id: Uuid,
        credit_score: &str,
        credit_rating: &str,
        risk_level: &str,
    ) -> AtlasResult<CreditProfile>;
    async fn update_profile_review_dates(
        &self,
        id: Uuid,
        last_review_date: Option<chrono::NaiveDate>,
        next_review_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<()>;
    async fn delete_profile(&self, org_id: Uuid, profile_number: &str) -> AtlasResult<()>;

    // Credit Limits
    async fn create_credit_limit(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        limit_type: &str,
        currency_code: Option<&str>,
        credit_limit: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditLimit>;

    async fn get_credit_limit(&self, id: Uuid) -> AtlasResult<Option<CreditLimit>>;
    async fn list_credit_limits(&self, profile_id: Uuid) -> AtlasResult<Vec<CreditLimit>>;
    async fn update_credit_limit_amount(&self, id: Uuid, credit_limit: &str) -> AtlasResult<CreditLimit>;
    async fn update_credit_limit_usage(&self, id: Uuid, used_amount: &str, available_amount: &str, hold_amount: &str) -> AtlasResult<()>;
    async fn set_temp_limit(&self, id: Uuid, temp_increase: &str, expiry: Option<chrono::NaiveDate>) -> AtlasResult<CreditLimit>;
    async fn delete_credit_limit(&self, id: Uuid) -> AtlasResult<()>;

    // Credit Check Rules
    async fn create_check_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        check_point: &str,
        check_type: &str,
        condition: serde_json::Value,
        action_on_failure: &str,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditCheckRule>;

    async fn get_check_rule(&self, id: Uuid) -> AtlasResult<Option<CreditCheckRule>>;
    async fn get_check_rule_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<CreditCheckRule>>;
    async fn list_check_rules(&self, org_id: Uuid) -> AtlasResult<Vec<CreditCheckRule>>;
    async fn delete_check_rule(&self, id: Uuid) -> AtlasResult<()>;

    // Credit Exposure
    async fn create_exposure(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        exposure_date: chrono::NaiveDate,
        currency_code: &str,
        open_receivables: &str,
        open_orders: &str,
        open_shipments: &str,
        open_invoices: &str,
        unapplied_cash: &str,
        on_hold_amount: &str,
        total_exposure: &str,
        credit_limit: &str,
        available_credit: &str,
        utilization_percent: &str,
    ) -> AtlasResult<CreditExposure>;

    async fn get_latest_exposure(&self, profile_id: Uuid) -> AtlasResult<Option<CreditExposure>>;
    async fn list_exposure_history(&self, profile_id: Uuid, limit: Option<i32>) -> AtlasResult<Vec<CreditExposure>>;

    // Credit Holds
    async fn create_hold(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        hold_number: &str,
        hold_type: &str,
        entity_type: &str,
        entity_id: Uuid,
        entity_number: Option<&str>,
        hold_amount: Option<&str>,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditHold>;

    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<CreditHold>>;
    async fn list_holds(&self, org_id: Uuid, status: Option<&str>, profile_id: Option<Uuid>) -> AtlasResult<Vec<CreditHold>>;
    async fn release_hold(&self, id: Uuid, released_by: Option<Uuid>, release_reason: Option<&str>) -> AtlasResult<CreditHold>;
    async fn override_hold(&self, id: Uuid, overridden_by: Option<Uuid>, override_reason: Option<&str>) -> AtlasResult<CreditHold>;

    // Credit Reviews
    async fn create_review(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        review_number: &str,
        review_type: &str,
        previous_credit_limit: Option<&str>,
        recommended_credit_limit: Option<&str>,
        previous_score: Option<&str>,
        previous_rating: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditReview>;

    async fn get_review(&self, id: Uuid) -> AtlasResult<Option<CreditReview>>;
    async fn list_reviews(&self, org_id: Uuid, status: Option<&str>, profile_id: Option<Uuid>) -> AtlasResult<Vec<CreditReview>>;
    async fn update_review_status(&self, id: Uuid, status: &str) -> AtlasResult<CreditReview>;
    async fn complete_review(
        &self,
        id: Uuid,
        new_score: Option<&str>,
        new_rating: Option<&str>,
        approved_credit_limit: Option<&str>,
        findings: Option<&str>,
        recommendations: Option<&str>,
        reviewer_id: Option<Uuid>,
        reviewer_name: Option<&str>,
    ) -> AtlasResult<CreditReview>;
    async fn approve_review(
        &self,
        id: Uuid,
        approver_id: Option<Uuid>,
        approver_name: Option<&str>,
    ) -> AtlasResult<CreditReview>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CreditManagementDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresCreditManagementRepository {
    pool: PgPool,
}

impl PostgresCreditManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CreditManagementRepository for PostgresCreditManagementRepository {
    // Scoring Models
    async fn create_scoring_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        model_type: &str,
        scoring_criteria: serde_json::Value,
        score_ranges: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditScoringModel> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_scoring_models
                (organization_id, code, name, description, model_type,
                 scoring_criteria, score_ranges, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(model_type).bind(&scoring_criteria).bind(&score_ranges)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_scoring_model(&row))
    }

    async fn get_scoring_model(&self, id: Uuid) -> AtlasResult<Option<CreditScoringModel>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_scoring_models WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_scoring_model(&r)))
    }

    async fn get_scoring_model_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CreditScoringModel>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_scoring_models WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_scoring_model(&r)))
    }

    async fn list_scoring_models(&self, org_id: Uuid) -> AtlasResult<Vec<CreditScoringModel>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.credit_scoring_models WHERE organization_id = $1 AND is_active = true ORDER BY name"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_scoring_model).collect())
    }

    async fn delete_scoring_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.credit_scoring_models WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Credit Profiles
    async fn create_profile(
        &self,
        org_id: Uuid,
        profile_number: &str,
        profile_name: &str,
        description: Option<&str>,
        profile_type: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        customer_group_id: Option<Uuid>,
        customer_group_name: Option<&str>,
        scoring_model_id: Option<Uuid>,
        review_frequency_days: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditProfile> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_profiles
                (organization_id, profile_number, profile_name, description,
                 profile_type, customer_id, customer_name,
                 customer_group_id, customer_group_name,
                 scoring_model_id, review_frequency_days, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(org_id).bind(profile_number).bind(profile_name).bind(description)
        .bind(profile_type).bind(customer_id).bind(customer_name)
        .bind(customer_group_id).bind(customer_group_name)
        .bind(scoring_model_id).bind(review_frequency_days).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_profile(&row))
    }

    async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<CreditProfile>> {
        let row = sqlx::query("SELECT * FROM _atlas.credit_profiles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_profile(&r)))
    }

    async fn get_profile_by_number(&self, org_id: Uuid, profile_number: &str) -> AtlasResult<Option<CreditProfile>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_profiles WHERE organization_id = $1 AND profile_number = $2"
        )
        .bind(org_id).bind(profile_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_profile(&r)))
    }

    async fn get_profile_by_customer(&self, org_id: Uuid, customer_id: Uuid) -> AtlasResult<Option<CreditProfile>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_profiles WHERE organization_id = $1 AND customer_id = $2 AND status = 'active' ORDER BY created_at DESC LIMIT 1"
        )
        .bind(org_id).bind(customer_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_profile(&r)))
    }

    async fn list_profiles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CreditProfile>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.credit_profiles WHERE organization_id = $1 AND status = $2 ORDER BY profile_name"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.credit_profiles WHERE organization_id = $1 ORDER BY profile_name"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
        };

        Ok(rows.iter().map(row_to_profile).collect())
    }

    async fn update_profile_status(&self, id: Uuid, status: &str) -> AtlasResult<CreditProfile> {
        let row = sqlx::query(
            "UPDATE _atlas.credit_profiles SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_profile(&row))
    }

    async fn update_profile_score(
        &self,
        id: Uuid,
        credit_score: &str,
        credit_rating: &str,
        risk_level: &str,
    ) -> AtlasResult<CreditProfile> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_profiles
            SET credit_score = $2, credit_rating = $3, risk_level = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(credit_score.parse::<f64>().ok())
        .bind(credit_rating)
        .bind(risk_level)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_profile(&row))
    }

    async fn update_profile_review_dates(
        &self,
        id: Uuid,
        last_review_date: Option<chrono::NaiveDate>,
        next_review_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.credit_profiles SET last_review_date = $2, next_review_date = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(last_review_date).bind(next_review_date)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_profile(&self, org_id: Uuid, profile_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.credit_profiles WHERE organization_id = $1 AND profile_number = $2")
            .bind(org_id).bind(profile_number).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Credit Limits
    async fn create_credit_limit(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        limit_type: &str,
        currency_code: Option<&str>,
        credit_limit: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditLimit> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_limits
                (organization_id, profile_id, limit_type, currency_code,
                 credit_limit, available_amount, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(org_id).bind(profile_id).bind(limit_type).bind(currency_code)
        .bind(credit_limit.parse::<f64>().unwrap_or(0.0))
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_credit_limit(&row))
    }

    async fn get_credit_limit(&self, id: Uuid) -> AtlasResult<Option<CreditLimit>> {
        let row = sqlx::query("SELECT * FROM _atlas.credit_limits WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_credit_limit(&r)))
    }

    async fn list_credit_limits(&self, profile_id: Uuid) -> AtlasResult<Vec<CreditLimit>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.credit_limits WHERE profile_id = $1 AND is_active = true ORDER BY limit_type, currency_code"
        )
        .bind(profile_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_credit_limit).collect())
    }

    async fn update_credit_limit_amount(&self, id: Uuid, credit_limit: &str) -> AtlasResult<CreditLimit> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_limits
            SET credit_limit = $2,
                available_amount = $2 - used_amount - hold_amount,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(credit_limit.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_credit_limit(&row))
    }

    async fn update_credit_limit_usage(&self, id: Uuid, used_amount: &str, available_amount: &str, hold_amount: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.credit_limits
            SET used_amount = $2, available_amount = $3, hold_amount = $4, updated_at = now()
            WHERE id = $1
            "#
        )
        .bind(id)
        .bind(used_amount.parse::<f64>().unwrap_or(0.0))
        .bind(available_amount.parse::<f64>().unwrap_or(0.0))
        .bind(hold_amount.parse::<f64>().unwrap_or(0.0))
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn set_temp_limit(&self, id: Uuid, temp_increase: &str, expiry: Option<chrono::NaiveDate>) -> AtlasResult<CreditLimit> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_limits
            SET temp_limit_increase = $2, temp_limit_expiry = $3,
                available_amount = credit_limit + $2 - used_amount - hold_amount,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(temp_increase.parse::<f64>().unwrap_or(0.0)).bind(expiry)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_credit_limit(&row))
    }

    async fn delete_credit_limit(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.credit_limits WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Credit Check Rules
    async fn create_check_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        check_point: &str,
        check_type: &str,
        condition: serde_json::Value,
        action_on_failure: &str,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditCheckRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_check_rules
                (organization_id, name, description, check_point, check_type,
                 condition, action_on_failure, priority,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(org_id).bind(name).bind(description)
        .bind(check_point).bind(check_type)
        .bind(&condition).bind(action_on_failure).bind(priority)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_check_rule(&row))
    }

    async fn get_check_rule(&self, id: Uuid) -> AtlasResult<Option<CreditCheckRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_check_rules WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_check_rule(&r)))
    }

    async fn get_check_rule_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<CreditCheckRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_check_rules WHERE organization_id = $1 AND name = $2 AND is_active = true"
        )
        .bind(org_id).bind(name)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_check_rule(&r)))
    }

    async fn list_check_rules(&self, org_id: Uuid) -> AtlasResult<Vec<CreditCheckRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.credit_check_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_check_rule).collect())
    }

    async fn delete_check_rule(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.credit_check_rules WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Credit Exposure
    async fn create_exposure(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        exposure_date: chrono::NaiveDate,
        currency_code: &str,
        open_receivables: &str,
        open_orders: &str,
        open_shipments: &str,
        open_invoices: &str,
        unapplied_cash: &str,
        on_hold_amount: &str,
        total_exposure: &str,
        credit_limit: &str,
        available_credit: &str,
        utilization_percent: &str,
    ) -> AtlasResult<CreditExposure> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_exposure
                (organization_id, profile_id, exposure_date, currency_code,
                 open_receivables, open_orders, open_shipments, open_invoices,
                 unapplied_cash, on_hold_amount, total_exposure,
                 credit_limit, available_credit, utilization_percent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (profile_id, exposure_date, currency_code)
            DO UPDATE SET
                open_receivables = EXCLUDED.open_receivables,
                open_orders = EXCLUDED.open_orders,
                open_shipments = EXCLUDED.open_shipments,
                open_invoices = EXCLUDED.open_invoices,
                unapplied_cash = EXCLUDED.unapplied_cash,
                on_hold_amount = EXCLUDED.on_hold_amount,
                total_exposure = EXCLUDED.total_exposure,
                credit_limit = EXCLUDED.credit_limit,
                available_credit = EXCLUDED.available_credit,
                utilization_percent = EXCLUDED.utilization_percent,
                updated_at = now()
            RETURNING *
            "#
        )
        .bind(org_id).bind(profile_id).bind(exposure_date).bind(currency_code)
        .bind(open_receivables.parse::<f64>().unwrap_or(0.0))
        .bind(open_orders.parse::<f64>().unwrap_or(0.0))
        .bind(open_shipments.parse::<f64>().unwrap_or(0.0))
        .bind(open_invoices.parse::<f64>().unwrap_or(0.0))
        .bind(unapplied_cash.parse::<f64>().unwrap_or(0.0))
        .bind(on_hold_amount.parse::<f64>().unwrap_or(0.0))
        .bind(total_exposure.parse::<f64>().unwrap_or(0.0))
        .bind(credit_limit.parse::<f64>().unwrap_or(0.0))
        .bind(available_credit.parse::<f64>().unwrap_or(0.0))
        .bind(utilization_percent.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_exposure(&row))
    }

    async fn get_latest_exposure(&self, profile_id: Uuid) -> AtlasResult<Option<CreditExposure>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_exposure WHERE profile_id = $1 ORDER BY exposure_date DESC LIMIT 1"
        )
        .bind(profile_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_exposure(&r)))
    }

    async fn list_exposure_history(&self, profile_id: Uuid, limit: Option<i32>) -> AtlasResult<Vec<CreditExposure>> {
        let limit_val = limit.unwrap_or(30);
        let rows = sqlx::query(
            "SELECT * FROM _atlas.credit_exposure WHERE profile_id = $1 ORDER BY exposure_date DESC LIMIT $2"
        )
        .bind(profile_id).bind(limit_val as i64)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_exposure).collect())
    }

    // Credit Holds
    async fn create_hold(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        hold_number: &str,
        hold_type: &str,
        entity_type: &str,
        entity_id: Uuid,
        entity_number: Option<&str>,
        hold_amount: Option<&str>,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditHold> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_holds
                (organization_id, profile_id, hold_number, hold_type,
                 entity_type, entity_id, entity_number, hold_amount,
                 reason, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(org_id).bind(profile_id).bind(hold_number).bind(hold_type)
        .bind(entity_type).bind(entity_id).bind(entity_number)
        .bind(hold_amount.and_then(|v| v.parse::<f64>().ok()))
        .bind(reason).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<CreditHold>> {
        let row = sqlx::query("SELECT * FROM _atlas.credit_holds WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_hold(&r)))
    }

    async fn list_holds(&self, org_id: Uuid, status: Option<&str>, profile_id: Option<Uuid>) -> AtlasResult<Vec<CreditHold>> {
        let rows = match (status, profile_id) {
            (Some(s), Some(pid)) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_holds WHERE organization_id = $1 AND status = $2 AND profile_id = $3 ORDER BY created_at DESC"
                )
                .bind(org_id).bind(s).bind(pid)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
            (Some(s), None) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_holds WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
                )
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
            (None, Some(pid)) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_holds WHERE organization_id = $1 AND profile_id = $2 ORDER BY created_at DESC"
                )
                .bind(org_id).bind(pid)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
            (None, None) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_holds WHERE organization_id = $1 ORDER BY created_at DESC"
                )
                .bind(org_id)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
        };

        Ok(rows.iter().map(row_to_hold).collect())
    }

    async fn release_hold(&self, id: Uuid, released_by: Option<Uuid>, release_reason: Option<&str>) -> AtlasResult<CreditHold> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_holds
            SET status = 'released', released_by = $2, released_at = now(), release_reason = $3, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(released_by).bind(release_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    async fn override_hold(&self, id: Uuid, overridden_by: Option<Uuid>, override_reason: Option<&str>) -> AtlasResult<CreditHold> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_holds
            SET status = 'overridden', overridden_by = $2, overridden_at = now(), override_reason = $3, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(overridden_by).bind(override_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    // Credit Reviews
    async fn create_review(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        review_number: &str,
        review_type: &str,
        previous_credit_limit: Option<&str>,
        recommended_credit_limit: Option<&str>,
        previous_score: Option<&str>,
        previous_rating: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditReview> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_reviews
                (organization_id, profile_id, review_number, review_type,
                 previous_credit_limit, recommended_credit_limit,
                 previous_score, previous_rating, due_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(org_id).bind(profile_id).bind(review_number).bind(review_type)
        .bind(previous_credit_limit.and_then(|v| v.parse::<f64>().ok()))
        .bind(recommended_credit_limit.and_then(|v| v.parse::<f64>().ok()))
        .bind(previous_score.and_then(|v| v.parse::<f64>().ok()))
        .bind(previous_rating)
        .bind(due_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_review(&row))
    }

    async fn get_review(&self, id: Uuid) -> AtlasResult<Option<CreditReview>> {
        let row = sqlx::query("SELECT * FROM _atlas.credit_reviews WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_review(&r)))
    }

    async fn list_reviews(&self, org_id: Uuid, status: Option<&str>, profile_id: Option<Uuid>) -> AtlasResult<Vec<CreditReview>> {
        let rows = match (status, profile_id) {
            (Some(s), Some(pid)) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_reviews WHERE organization_id = $1 AND status = $2 AND profile_id = $3 ORDER BY created_at DESC"
                )
                .bind(org_id).bind(s).bind(pid)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
            (Some(s), None) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_reviews WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
                )
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
            (None, Some(pid)) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_reviews WHERE organization_id = $1 AND profile_id = $2 ORDER BY created_at DESC"
                )
                .bind(org_id).bind(pid)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
            (None, None) => {
                sqlx::query(
                    "SELECT * FROM _atlas.credit_reviews WHERE organization_id = $1 ORDER BY created_at DESC"
                )
                .bind(org_id)
                .fetch_all(&self.pool).await
                .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
            }
        };

        Ok(rows.iter().map(row_to_review).collect())
    }

    async fn update_review_status(&self, id: Uuid, status: &str) -> AtlasResult<CreditReview> {
        let row = sqlx::query(
            "UPDATE _atlas.credit_reviews SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_review(&row))
    }

    async fn complete_review(
        &self,
        id: Uuid,
        new_score: Option<&str>,
        new_rating: Option<&str>,
        approved_credit_limit: Option<&str>,
        findings: Option<&str>,
        recommendations: Option<&str>,
        reviewer_id: Option<Uuid>,
        reviewer_name: Option<&str>,
    ) -> AtlasResult<CreditReview> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_reviews
            SET status = 'completed',
                new_score = $2, new_rating = $3,
                approved_credit_limit = $4,
                findings = $5, recommendations = $6,
                reviewer_id = $7, reviewer_name = $8,
                reviewed_at = now(), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(new_score.and_then(|v| v.parse::<f64>().ok()))
        .bind(new_rating)
        .bind(approved_credit_limit.and_then(|v| v.parse::<f64>().ok()))
        .bind(findings).bind(recommendations)
        .bind(reviewer_id).bind(reviewer_name)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_review(&row))
    }

    async fn approve_review(
        &self,
        id: Uuid,
        approver_id: Option<Uuid>,
        approver_name: Option<&str>,
    ) -> AtlasResult<CreditReview> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.credit_reviews
            SET status = 'approved',
                approver_id = $2, approver_name = $3,
                approved_at = now(), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(approver_id).bind(approver_name)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_review(&row))
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CreditManagementDashboard> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_profiles,
                COUNT(*) FILTER (WHERE status = 'active') as active_profiles,
                COUNT(*) FILTER (WHERE status IN ('blocked', 'suspended')) as blocked_profiles,
                COALESCE(SUM(l.credit_limit), 0) as total_credit_limit,
                COALESCE(SUM(l.used_amount), 0) as total_exposure,
                COALESCE(SUM(l.available_amount), 0) as total_available
            FROM _atlas.credit_profiles p
            LEFT JOIN _atlas.credit_limits l ON l.profile_id = p.id AND l.is_active = true AND l.limit_type = 'overall'
            WHERE p.organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let total_limit: f64 = row.try_get("total_credit_limit").unwrap_or(0.0);
        let total_exposure: f64 = row.try_get("total_exposure").unwrap_or(0.0);

        // Get holds count
        let hold_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.credit_holds WHERE organization_id = $1 AND status = 'active'"
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        // Get pending reviews
        let review_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status IN ('pending', 'in_review')) as pending,
                COUNT(*) FILTER (WHERE status IN ('pending', 'in_review') AND due_date < CURRENT_DATE) as overdue
            FROM _atlas.credit_reviews
            WHERE organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let avg_util = if total_limit > 0.0 {
            total_exposure / total_limit * 100.0
        } else {
            0.0
        };

        Ok(CreditManagementDashboard {
            total_profiles: row.get::<i64, _>("total_profiles") as i32,
            active_profiles: row.get::<i64, _>("active_profiles") as i32,
            blocked_profiles: row.get::<i64, _>("blocked_profiles") as i32,
            total_credit_limit: format!("{:.2}", total_limit),
            total_exposure: format!("{:.2}", total_exposure),
            total_available: format!("{:.2}", row.try_get::<f64, _>("total_available").unwrap_or(0.0)),
            active_holds: hold_row.get::<i64, _>("cnt") as i32,
            pending_reviews: review_row.get::<i64, _>("pending") as i32,
            overdue_reviews: review_row.get::<i64, _>("overdue") as i32,
            average_utilization: format!("{:.2}", avg_util),
        })
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

use sqlx::Row;

fn row_to_scoring_model(row: &sqlx::postgres::PgRow) -> CreditScoringModel {
    CreditScoringModel {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        model_type: row.get("model_type"),
        scoring_criteria: row.get("scoring_criteria"),
        score_ranges: row.get("score_ranges"),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_profile(row: &sqlx::postgres::PgRow) -> CreditProfile {
    CreditProfile {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        profile_number: row.get("profile_number"),
        profile_name: row.get("profile_name"),
        description: row.get("description"),
        profile_type: row.get("profile_type"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        customer_group_id: row.get("customer_group_id"),
        customer_group_name: row.get("customer_group_name"),
        scoring_model_id: row.get("scoring_model_id"),
        credit_score: row.try_get("credit_score").ok().map(|v: f64| format!("{:.2}", v)),
        credit_rating: row.get("credit_rating"),
        risk_level: row.get("risk_level"),
        status: row.get("status"),
        review_frequency_days: row.get("review_frequency_days"),
        last_review_date: row.get("last_review_date"),
        next_review_date: row.get("next_review_date"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_credit_limit(row: &sqlx::postgres::PgRow) -> CreditLimit {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: f64 = row.try_get(col).unwrap_or(0.0);
        format!("{:.2}", v)
    }
    CreditLimit {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        profile_id: row.get("profile_id"),
        limit_type: row.get("limit_type"),
        currency_code: row.get("currency_code"),
        credit_limit: get_num(row, "credit_limit"),
        temp_limit_increase: get_num(row, "temp_limit_increase"),
        temp_limit_expiry: row.get("temp_limit_expiry"),
        used_amount: get_num(row, "used_amount"),
        available_amount: get_num(row, "available_amount"),
        hold_amount: get_num(row, "hold_amount"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        is_active: row.get("is_active"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_check_rule(row: &sqlx::postgres::PgRow) -> CreditCheckRule {
    CreditCheckRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        name: row.get("name"),
        description: row.get("description"),
        check_point: row.get("check_point"),
        check_type: row.get("check_type"),
        condition: row.get("condition"),
        action_on_failure: row.get("action_on_failure"),
        priority: row.get("priority"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_exposure(row: &sqlx::postgres::PgRow) -> CreditExposure {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: f64 = row.try_get(col).unwrap_or(0.0);
        format!("{:.2}", v)
    }
    CreditExposure {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        profile_id: row.get("profile_id"),
        exposure_date: row.get("exposure_date"),
        open_receivables: get_num(row, "open_receivables"),
        open_orders: get_num(row, "open_orders"),
        open_shipments: get_num(row, "open_shipments"),
        open_invoices: get_num(row, "open_invoices"),
        unapplied_cash: get_num(row, "unapplied_cash"),
        on_hold_amount: get_num(row, "on_hold_amount"),
        total_exposure: get_num(row, "total_exposure"),
        credit_limit: get_num(row, "credit_limit"),
        available_credit: get_num(row, "available_credit"),
        utilization_percent: get_num(row, "utilization_percent"),
        currency_code: row.get("currency_code"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_hold(row: &sqlx::postgres::PgRow) -> CreditHold {
    CreditHold {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        profile_id: row.get("profile_id"),
        hold_number: row.get("hold_number"),
        hold_type: row.get("hold_type"),
        entity_type: row.get("entity_type"),
        entity_id: row.get("entity_id"),
        entity_number: row.get("entity_number"),
        hold_amount: row.try_get("hold_amount").ok().map(|v: f64| format!("{:.2}", v)),
        reason: row.get("reason"),
        status: row.get("status"),
        released_by: row.get("released_by"),
        released_at: row.get("released_at"),
        release_reason: row.get("release_reason"),
        overridden_by: row.get("overridden_by"),
        overridden_at: row.get("overridden_at"),
        override_reason: row.get("override_reason"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_review(row: &sqlx::postgres::PgRow) -> CreditReview {
    CreditReview {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        profile_id: row.get("profile_id"),
        review_number: row.get("review_number"),
        review_type: row.get("review_type"),
        status: row.get("status"),
        previous_credit_limit: row.try_get("previous_credit_limit").ok().map(|v: f64| format!("{:.2}", v)),
        recommended_credit_limit: row.try_get("recommended_credit_limit").ok().map(|v: f64| format!("{:.2}", v)),
        approved_credit_limit: row.try_get("approved_credit_limit").ok().map(|v: f64| format!("{:.2}", v)),
        previous_score: row.try_get("previous_score").ok().map(|v: f64| format!("{:.2}", v)),
        new_score: row.try_get("new_score").ok().map(|v: f64| format!("{:.2}", v)),
        previous_rating: row.get("previous_rating"),
        new_rating: row.get("new_rating"),
        findings: row.get("findings"),
        recommendations: row.get("recommendations"),
        reviewer_id: row.get("reviewer_id"),
        reviewer_name: row.get("reviewer_name"),
        reviewed_at: row.get("reviewed_at"),
        approver_id: row.get("approver_id"),
        approver_name: row.get("approver_name"),
        approved_at: row.get("approved_at"),
        rejected_reason: row.get("rejected_reason"),
        due_date: row.get("due_date"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
