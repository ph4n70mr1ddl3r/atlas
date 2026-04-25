//! Lead and Opportunity Repository
//!
//! PostgreSQL storage for sales lead and opportunity data.

use atlas_shared::{
    LeadSource, LeadRatingModel, SalesLead, OpportunityStage,
    SalesOpportunity, OpportunityLine, SalesActivity,
    OpportunityStageHistory, SalesPipelineDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for Lead and Opportunity data storage
#[async_trait]
pub trait LeadOpportunityRepository: Send + Sync {
    // Lead Sources
    async fn create_lead_source(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeadSource>;
    async fn get_lead_source_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LeadSource>>;
    async fn list_lead_sources(&self, org_id: Uuid) -> AtlasResult<Vec<LeadSource>>;
    async fn delete_lead_source(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Lead Rating Models
    async fn create_lead_rating_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        scoring_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeadRatingModel>;
    async fn get_lead_rating_model(&self, id: Uuid) -> AtlasResult<Option<LeadRatingModel>>;
    async fn get_lead_rating_model_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LeadRatingModel>>;
    async fn list_lead_rating_models(&self, org_id: Uuid) -> AtlasResult<Vec<LeadRatingModel>>;
    async fn delete_lead_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Sales Leads
    async fn create_lead(
        &self,
        org_id: Uuid,
        lead_number: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        company: Option<&str>,
        title: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        website: Option<&str>,
        industry: Option<&str>,
        lead_source_id: Option<Uuid>,
        lead_source_name: Option<&str>,
        lead_rating_model_id: Option<Uuid>,
        estimated_value: &str,
        currency_code: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesLead>;
    async fn get_lead(&self, id: Uuid) -> AtlasResult<Option<SalesLead>>;
    async fn get_lead_by_number(&self, org_id: Uuid, lead_number: &str) -> AtlasResult<Option<SalesLead>>;
    async fn list_leads(&self, org_id: Uuid, status: Option<&str>, owner_id: Option<Uuid>) -> AtlasResult<Vec<SalesLead>>;
    async fn update_lead_status(&self, id: Uuid, status: &str) -> AtlasResult<SalesLead>;
    async fn update_lead_score(&self, id: Uuid, score: &str, rating: &str) -> AtlasResult<SalesLead>;
    async fn convert_lead(
        &self,
        id: Uuid,
        opportunity_id: Option<Uuid>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<SalesLead>;
    async fn delete_lead(&self, org_id: Uuid, lead_number: &str) -> AtlasResult<()>;

    // Opportunity Stages
    async fn create_opportunity_stage(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        probability: &str,
        display_order: i32,
        is_won: bool,
        is_lost: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<OpportunityStage>;
    async fn get_opportunity_stage(&self, id: Uuid) -> AtlasResult<Option<OpportunityStage>>;
    async fn get_opportunity_stage_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<OpportunityStage>>;
    async fn list_opportunity_stages(&self, org_id: Uuid) -> AtlasResult<Vec<OpportunityStage>>;
    async fn delete_opportunity_stage(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Sales Opportunities
    async fn create_opportunity(
        &self,
        org_id: Uuid,
        opportunity_number: &str,
        name: &str,
        description: Option<&str>,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        lead_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        stage_name: Option<&str>,
        amount: &str,
        currency_code: &str,
        probability: &str,
        expected_close_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesOpportunity>;
    async fn get_opportunity(&self, id: Uuid) -> AtlasResult<Option<SalesOpportunity>>;
    async fn get_opportunity_by_number(&self, org_id: Uuid, opportunity_number: &str) -> AtlasResult<Option<SalesOpportunity>>;
    async fn list_opportunities(&self, org_id: Uuid, status: Option<&str>, owner_id: Option<Uuid>, stage_id: Option<Uuid>) -> AtlasResult<Vec<SalesOpportunity>>;
    async fn update_opportunity_stage(
        &self,
        id: Uuid,
        stage_id: Option<Uuid>,
        stage_name: Option<&str>,
        probability: &str,
        weighted_amount: &str,
    ) -> AtlasResult<SalesOpportunity>;
    async fn update_opportunity_amount(&self, id: Uuid, amount: &str, weighted_amount: &str) -> AtlasResult<SalesOpportunity>;
    async fn close_opportunity_won(&self, id: Uuid) -> AtlasResult<SalesOpportunity>;
    async fn close_opportunity_lost(&self, id: Uuid, lost_reason: Option<&str>) -> AtlasResult<SalesOpportunity>;
    async fn delete_opportunity(&self, org_id: Uuid, opportunity_number: &str) -> AtlasResult<()>;

    // Opportunity Lines
    async fn add_opportunity_line(
        &self,
        org_id: Uuid,
        opportunity_id: Uuid,
        line_number: i32,
        product_name: &str,
        product_code: Option<&str>,
        description: Option<&str>,
        quantity: &str,
        unit_price: &str,
        line_amount: &str,
        discount_percent: &str,
    ) -> AtlasResult<OpportunityLine>;
    async fn list_opportunity_lines(&self, opportunity_id: Uuid) -> AtlasResult<Vec<OpportunityLine>>;
    async fn delete_opportunity_line(&self, id: Uuid) -> AtlasResult<()>;

    // Sales Activities
    async fn create_activity(
        &self,
        org_id: Uuid,
        subject: &str,
        description: Option<&str>,
        activity_type: &str,
        priority: &str,
        lead_id: Option<Uuid>,
        opportunity_id: Option<Uuid>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        end_at: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesActivity>;
    async fn get_activity(&self, id: Uuid) -> AtlasResult<Option<SalesActivity>>;
    async fn list_activities(&self, org_id: Uuid, lead_id: Option<Uuid>, opportunity_id: Option<Uuid>) -> AtlasResult<Vec<SalesActivity>>;
    async fn complete_activity(&self, id: Uuid, outcome: Option<&str>) -> AtlasResult<SalesActivity>;
    async fn cancel_activity(&self, id: Uuid) -> AtlasResult<SalesActivity>;
    async fn delete_activity(&self, id: Uuid) -> AtlasResult<()>;

    // Stage History
    async fn add_stage_history(
        &self,
        org_id: Uuid,
        opportunity_id: Uuid,
        from_stage: Option<&str>,
        to_stage: &str,
        changed_by: Option<Uuid>,
        changed_by_name: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<OpportunityStageHistory>;
    async fn list_stage_history(&self, opportunity_id: Uuid) -> AtlasResult<Vec<OpportunityStageHistory>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SalesPipelineDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresLeadOpportunityRepository {
    pool: PgPool,
}

impl PostgresLeadOpportunityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeadOpportunityRepository for PostgresLeadOpportunityRepository {
    // Lead Sources
    async fn create_lead_source(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeadSource> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.lead_sources (organization_id, code, name, description, created_by)
               VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_lead_source(&row))
    }

    async fn get_lead_source_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LeadSource>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.lead_sources WHERE organization_id = $1 AND code = $2 AND is_active = true",
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_lead_source(&r)))
    }

    async fn list_lead_sources(&self, org_id: Uuid) -> AtlasResult<Vec<LeadSource>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.lead_sources WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_lead_source).collect())
    }

    async fn delete_lead_source(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.lead_sources WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Lead Rating Models
    async fn create_lead_rating_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        scoring_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeadRatingModel> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.lead_rating_models (organization_id, code, name, description, scoring_criteria, created_by)
               VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(&scoring_criteria).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_lead_rating_model(&row))
    }

    async fn get_lead_rating_model(&self, id: Uuid) -> AtlasResult<Option<LeadRatingModel>> {
        let row = sqlx::query("SELECT * FROM _atlas.lead_rating_models WHERE id = $1 AND is_active = true")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_lead_rating_model(&r)))
    }

    async fn get_lead_rating_model_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LeadRatingModel>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.lead_rating_models WHERE organization_id = $1 AND code = $2 AND is_active = true",
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_lead_rating_model(&r)))
    }

    async fn list_lead_rating_models(&self, org_id: Uuid) -> AtlasResult<Vec<LeadRatingModel>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.lead_rating_models WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_lead_rating_model).collect())
    }

    async fn delete_lead_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.lead_rating_models WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Sales Leads
    async fn create_lead(
        &self,
        org_id: Uuid,
        lead_number: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        company: Option<&str>,
        title: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        website: Option<&str>,
        industry: Option<&str>,
        lead_source_id: Option<Uuid>,
        lead_source_name: Option<&str>,
        lead_rating_model_id: Option<Uuid>,
        estimated_value: &str,
        currency_code: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesLead> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sales_leads
                (organization_id, lead_number, first_name, last_name, company, title,
                 email, phone, website, industry, lead_source_id, lead_source_name,
                 lead_rating_model_id, estimated_value, currency_code, owner_id, owner_name,
                 notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
               RETURNING *"#,
        )
        .bind(org_id).bind(lead_number).bind(first_name).bind(last_name)
        .bind(company).bind(title).bind(email).bind(phone).bind(website)
        .bind(industry).bind(lead_source_id).bind(lead_source_name)
        .bind(lead_rating_model_id)
        .bind(estimated_value.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(owner_id).bind(owner_name)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_lead(&row))
    }

    async fn get_lead(&self, id: Uuid) -> AtlasResult<Option<SalesLead>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_leads WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_lead(&r)))
    }

    async fn get_lead_by_number(&self, org_id: Uuid, lead_number: &str) -> AtlasResult<Option<SalesLead>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sales_leads WHERE organization_id = $1 AND lead_number = $2",
        )
        .bind(org_id).bind(lead_number).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_lead(&r)))
    }

    async fn list_leads(&self, org_id: Uuid, status: Option<&str>, owner_id: Option<Uuid>) -> AtlasResult<Vec<SalesLead>> {
        let rows = match (status, owner_id) {
            (Some(s), Some(oid)) => sqlx::query(
                "SELECT * FROM _atlas.sales_leads WHERE organization_id = $1 AND status = $2 AND owner_id = $3 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).bind(oid).fetch_all(&self.pool).await,
            (Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.sales_leads WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, Some(oid)) => sqlx::query(
                "SELECT * FROM _atlas.sales_leads WHERE organization_id = $1 AND owner_id = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(oid).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.sales_leads WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_lead).collect())
    }

    async fn update_lead_status(&self, id: Uuid, status: &str) -> AtlasResult<SalesLead> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_leads SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_lead(&row))
    }

    async fn update_lead_score(&self, id: Uuid, score: &str, rating: &str) -> AtlasResult<SalesLead> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_leads SET lead_score = $2, lead_rating = $3, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(score.parse::<f64>().unwrap_or(0.0)).bind(rating)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_lead(&row))
    }

    async fn convert_lead(
        &self,
        id: Uuid,
        opportunity_id: Option<Uuid>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<SalesLead> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_leads
               SET status = 'converted', converted_opportunity_id = $2,
                   converted_customer_id = $3, converted_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(opportunity_id).bind(customer_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_lead(&row))
    }

    async fn delete_lead(&self, org_id: Uuid, lead_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.sales_leads WHERE organization_id = $1 AND lead_number = $2")
            .bind(org_id).bind(lead_number).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Opportunity Stages
    async fn create_opportunity_stage(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        probability: &str,
        display_order: i32,
        is_won: bool,
        is_lost: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<OpportunityStage> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.opportunity_stages
                (organization_id, code, name, description, probability, display_order, is_won, is_lost, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(probability.parse::<f64>().unwrap_or(0.0))
        .bind(display_order).bind(is_won).bind(is_lost).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_stage(&row))
    }

    async fn get_opportunity_stage(&self, id: Uuid) -> AtlasResult<Option<OpportunityStage>> {
        let row = sqlx::query("SELECT * FROM _atlas.opportunity_stages WHERE id = $1 AND is_active = true")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_stage(&r)))
    }

    async fn get_opportunity_stage_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<OpportunityStage>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.opportunity_stages WHERE organization_id = $1 AND code = $2 AND is_active = true",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_stage(&r)))
    }

    async fn list_opportunity_stages(&self, org_id: Uuid) -> AtlasResult<Vec<OpportunityStage>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.opportunity_stages WHERE organization_id = $1 AND is_active = true ORDER BY display_order",
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_stage).collect())
    }

    async fn delete_opportunity_stage(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.opportunity_stages WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Sales Opportunities
    async fn create_opportunity(
        &self,
        org_id: Uuid,
        opportunity_number: &str,
        name: &str,
        description: Option<&str>,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        lead_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        stage_name: Option<&str>,
        amount: &str,
        currency_code: &str,
        probability: &str,
        expected_close_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesOpportunity> {
        let amount_val: f64 = amount.parse().unwrap_or(0.0);
        let prob_val: f64 = probability.parse().unwrap_or(0.0);
        let weighted = amount_val * prob_val / 100.0;
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sales_opportunities
                (organization_id, opportunity_number, name, description,
                 customer_id, customer_name, lead_id, stage_id, stage_name,
                 amount, currency_code, probability, weighted_amount,
                 expected_close_date, owner_id, owner_name,
                 contact_id, contact_name, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
               RETURNING *"#,
        )
        .bind(org_id).bind(opportunity_number).bind(name).bind(description)
        .bind(customer_id).bind(customer_name).bind(lead_id).bind(stage_id).bind(stage_name)
        .bind(amount_val).bind(currency_code).bind(prob_val).bind(weighted)
        .bind(expected_close_date).bind(owner_id).bind(owner_name)
        .bind(contact_id).bind(contact_name).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_opportunity(&row))
    }

    async fn get_opportunity(&self, id: Uuid) -> AtlasResult<Option<SalesOpportunity>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_opportunities WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_opportunity(&r)))
    }

    async fn get_opportunity_by_number(&self, org_id: Uuid, opportunity_number: &str) -> AtlasResult<Option<SalesOpportunity>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sales_opportunities WHERE organization_id = $1 AND opportunity_number = $2",
        )
        .bind(org_id).bind(opportunity_number).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_opportunity(&r)))
    }

    async fn list_opportunities(&self, org_id: Uuid, status: Option<&str>, owner_id: Option<Uuid>, stage_id: Option<Uuid>) -> AtlasResult<Vec<SalesOpportunity>> {
        let rows = match (status, owner_id, stage_id) {
            (Some(s), Some(oid), None) => sqlx::query(
                "SELECT * FROM _atlas.sales_opportunities WHERE organization_id = $1 AND status = $2 AND owner_id = $3 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).bind(oid).fetch_all(&self.pool).await,
            (Some(s), None, None) => sqlx::query(
                "SELECT * FROM _atlas.sales_opportunities WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, Some(oid), None) => sqlx::query(
                "SELECT * FROM _atlas.sales_opportunities WHERE organization_id = $1 AND owner_id = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(oid).fetch_all(&self.pool).await,
            (None, None, Some(sid)) => sqlx::query(
                "SELECT * FROM _atlas.sales_opportunities WHERE organization_id = $1 AND stage_id = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(sid).fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.sales_opportunities WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_opportunity).collect())
    }

    async fn update_opportunity_stage(
        &self,
        id: Uuid,
        stage_id: Option<Uuid>,
        stage_name: Option<&str>,
        probability: &str,
        weighted_amount: &str,
    ) -> AtlasResult<SalesOpportunity> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_opportunities
               SET stage_id = $2, stage_name = $3, probability = $4, weighted_amount = $5, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(stage_id).bind(stage_name)
        .bind(probability.parse::<f64>().unwrap_or(0.0))
        .bind(weighted_amount.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_opportunity(&row))
    }

    async fn update_opportunity_amount(&self, id: Uuid, amount: &str, weighted_amount: &str) -> AtlasResult<SalesOpportunity> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_opportunities SET amount = $2, weighted_amount = $3, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(amount.parse::<f64>().unwrap_or(0.0)).bind(weighted_amount.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_opportunity(&row))
    }

    async fn close_opportunity_won(&self, id: Uuid) -> AtlasResult<SalesOpportunity> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_opportunities
               SET status = 'won', probability = 100, weighted_amount = amount,
                   actual_close_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_opportunity(&row))
    }

    async fn close_opportunity_lost(&self, id: Uuid, lost_reason: Option<&str>) -> AtlasResult<SalesOpportunity> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_opportunities
               SET status = 'lost', probability = 0, weighted_amount = 0,
                   lost_reason = $2, actual_close_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(lost_reason).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_opportunity(&row))
    }

    async fn delete_opportunity(&self, org_id: Uuid, opportunity_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.sales_opportunities WHERE organization_id = $1 AND opportunity_number = $2")
            .bind(org_id).bind(opportunity_number).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Opportunity Lines
    async fn add_opportunity_line(
        &self,
        org_id: Uuid,
        opportunity_id: Uuid,
        line_number: i32,
        product_name: &str,
        product_code: Option<&str>,
        description: Option<&str>,
        quantity: &str,
        unit_price: &str,
        line_amount: &str,
        discount_percent: &str,
    ) -> AtlasResult<OpportunityLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.opportunity_lines
                (organization_id, opportunity_id, line_number, product_name, product_code,
                 description, quantity, unit_price, line_amount, discount_percent)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"#,
        )
        .bind(org_id).bind(opportunity_id).bind(line_number).bind(product_name)
        .bind(product_code).bind(description)
        .bind(quantity.parse::<f64>().unwrap_or(1.0))
        .bind(unit_price.parse::<f64>().unwrap_or(0.0))
        .bind(line_amount.parse::<f64>().unwrap_or(0.0))
        .bind(discount_percent.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_opportunity_line(&row))
    }

    async fn list_opportunity_lines(&self, opportunity_id: Uuid) -> AtlasResult<Vec<OpportunityLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.opportunity_lines WHERE opportunity_id = $1 ORDER BY line_number",
        )
        .bind(opportunity_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_opportunity_line).collect())
    }

    async fn delete_opportunity_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.opportunity_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Sales Activities
    async fn create_activity(
        &self,
        org_id: Uuid,
        subject: &str,
        description: Option<&str>,
        activity_type: &str,
        priority: &str,
        lead_id: Option<Uuid>,
        opportunity_id: Option<Uuid>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        end_at: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesActivity> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sales_activities
                (organization_id, subject, description, activity_type, priority,
                 lead_id, opportunity_id, contact_id, contact_name,
                 owner_id, owner_name, start_at, end_at, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) RETURNING *"#,
        )
        .bind(org_id).bind(subject).bind(description).bind(activity_type).bind(priority)
        .bind(lead_id).bind(opportunity_id).bind(contact_id).bind(contact_name)
        .bind(owner_id).bind(owner_name).bind(start_at).bind(end_at).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_activity(&row))
    }

    async fn get_activity(&self, id: Uuid) -> AtlasResult<Option<SalesActivity>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_activities WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_activity(&r)))
    }

    async fn list_activities(&self, org_id: Uuid, lead_id: Option<Uuid>, opportunity_id: Option<Uuid>) -> AtlasResult<Vec<SalesActivity>> {
        let rows = match (lead_id, opportunity_id) {
            (Some(lid), None) => sqlx::query(
                "SELECT * FROM _atlas.sales_activities WHERE organization_id = $1 AND lead_id = $2 ORDER BY start_at DESC",
            ).bind(org_id).bind(lid).fetch_all(&self.pool).await,
            (None, Some(oid)) => sqlx::query(
                "SELECT * FROM _atlas.sales_activities WHERE organization_id = $1 AND opportunity_id = $2 ORDER BY start_at DESC",
            ).bind(org_id).bind(oid).fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.sales_activities WHERE organization_id = $1 ORDER BY start_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_activity).collect())
    }

    async fn complete_activity(&self, id: Uuid, outcome: Option<&str>) -> AtlasResult<SalesActivity> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_activities SET status = 'completed', outcome = $2, completed_at = now(), updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(outcome).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_activity(&row))
    }

    async fn cancel_activity(&self, id: Uuid) -> AtlasResult<SalesActivity> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_activities SET status = 'cancelled', updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_activity(&row))
    }

    async fn delete_activity(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.sales_activities WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Stage History
    async fn add_stage_history(
        &self,
        org_id: Uuid,
        opportunity_id: Uuid,
        from_stage: Option<&str>,
        to_stage: &str,
        changed_by: Option<Uuid>,
        changed_by_name: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<OpportunityStageHistory> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.opportunity_stage_history
                (organization_id, opportunity_id, from_stage, to_stage, changed_by, changed_by_name, notes)
               VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *"#,
        )
        .bind(org_id).bind(opportunity_id).bind(from_stage).bind(to_stage)
        .bind(changed_by).bind(changed_by_name).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_stage_history(&row))
    }

    async fn list_stage_history(&self, opportunity_id: Uuid) -> AtlasResult<Vec<OpportunityStageHistory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.opportunity_stage_history WHERE opportunity_id = $1 ORDER BY changed_at DESC",
        )
        .bind(opportunity_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_stage_history).collect())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SalesPipelineDashboard> {
        // Lead counts
        let lead_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'new') as new_leads,
                COUNT(*) FILTER (WHERE status = 'qualified') as qualified,
                COUNT(*) FILTER (WHERE status = 'converted') as converted
               FROM _atlas.sales_leads WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        // Opportunity counts and values
        let opp_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'open') as open_opp,
                COUNT(*) FILTER (WHERE status = 'won') as won,
                COUNT(*) FILTER (WHERE status = 'lost') as lost,
                COALESCE(SUM(amount), 0) as total_pipeline,
                COALESCE(SUM(weighted_amount), 0) as weighted_pipeline,
                COALESCE(SUM(amount) FILTER (WHERE status = 'won'), 0) as won_value
               FROM _atlas.sales_opportunities WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let total_leads: i64 = lead_row.get("total");
        let total_opp: i64 = opp_row.get("total");
        let won: i64 = opp_row.get("won");
        let won_value: f64 = opp_row.try_get("won_value").unwrap_or(0.0);
        let avg_deal = if won > 0 { won_value / won as f64 } else { 0.0 };
        let win_rate = if total_opp > 0 { (won as f64 / total_opp as f64) * 100.0 } else { 0.0 };

        // By stage
        let stage_rows = sqlx::query(
            r#"SELECT stage_name, COUNT(*) as cnt, COALESCE(SUM(amount), 0) as total
               FROM _atlas.sales_opportunities
               WHERE organization_id = $1 AND status = 'open'
               GROUP BY stage_name ORDER BY stage_name"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        let by_stage: serde_json::Value = stage_rows.iter().map(|r| {
            serde_json::json!({
                "stage": r.get::<Option<&str>, _>("stage_name").unwrap_or("Unknown"),
                "count": r.get::<i64, _>("cnt"),
                "value": format!("{:.2}", r.get::<f64, _>("total")),
            })
        }).collect();

        // By owner
        let owner_rows = sqlx::query(
            r#"SELECT owner_name, COUNT(*) as cnt, COALESCE(SUM(amount), 0) as total
               FROM _atlas.sales_opportunities
               WHERE organization_id = $1 AND status = 'open'
               GROUP BY owner_name ORDER BY total DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let by_owner: serde_json::Value = owner_rows.iter().map(|r| {
            serde_json::json!({
                "owner": r.get::<Option<&str>, _>("owner_name").unwrap_or("Unassigned"),
                "count": r.get::<i64, _>("cnt"),
                "value": format!("{:.2}", r.get::<f64, _>("total")),
            })
        }).collect();

        Ok(SalesPipelineDashboard {
            total_leads: total_leads as i32,
            new_leads: lead_row.get::<i64, _>("new_leads") as i32,
            qualified_leads: lead_row.get::<i64, _>("qualified") as i32,
            converted_leads: lead_row.get::<i64, _>("converted") as i32,
            total_opportunities: total_opp as i32,
            open_opportunities: opp_row.get::<i64, _>("open_opp") as i32,
            won_opportunities: won as i32,
            lost_opportunities: opp_row.get::<i64, _>("lost") as i32,
            total_pipeline_value: format!("{:.2}", opp_row.try_get::<f64, _>("total_pipeline").unwrap_or(0.0)),
            weighted_pipeline_value: format!("{:.2}", opp_row.try_get::<f64, _>("weighted_pipeline").unwrap_or(0.0)),
            total_won_value: format!("{:.2}", won_value),
            average_deal_size: format!("{:.2}", avg_deal),
            win_rate: format!("{:.1}", win_rate),
            by_stage,
            by_owner,
        })
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

use sqlx::Row;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_lead_source(row: &sqlx::postgres::PgRow) -> LeadSource {
    LeadSource {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_lead_rating_model(row: &sqlx::postgres::PgRow) -> LeadRatingModel {
    LeadRatingModel {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        scoring_criteria: row.get("scoring_criteria"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_lead(row: &sqlx::postgres::PgRow) -> SalesLead {
    SalesLead {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        lead_number: row.get("lead_number"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        company: row.get("company"),
        title: row.get("title"),
        email: row.get("email"),
        phone: row.get("phone"),
        website: row.get("website"),
        industry: row.get("industry"),
        lead_source_id: row.get("lead_source_id"),
        lead_source_name: row.get("lead_source_name"),
        lead_rating_model_id: row.get("lead_rating_model_id"),
        lead_score: get_num(row, "lead_score"),
        lead_rating: row.get("lead_rating"),
        estimated_value: get_num(row, "estimated_value"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        converted_opportunity_id: row.get("converted_opportunity_id"),
        converted_customer_id: row.get("converted_customer_id"),
        converted_at: row.get("converted_at"),
        notes: row.get("notes"),
        address: row.get("address"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_stage(row: &sqlx::postgres::PgRow) -> OpportunityStage {
    OpportunityStage {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        probability: get_num(row, "probability"),
        display_order: row.get("display_order"),
        is_won: row.get("is_won"),
        is_lost: row.get("is_lost"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_opportunity(row: &sqlx::postgres::PgRow) -> SalesOpportunity {
    SalesOpportunity {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        opportunity_number: row.get("opportunity_number"),
        name: row.get("name"),
        description: row.get("description"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        lead_id: row.get("lead_id"),
        stage_id: row.get("stage_id"),
        stage_name: row.get("stage_name"),
        amount: get_num(row, "amount"),
        currency_code: row.get("currency_code"),
        probability: get_num(row, "probability"),
        weighted_amount: get_num(row, "weighted_amount"),
        expected_close_date: row.get("expected_close_date"),
        actual_close_date: row.get("actual_close_date"),
        status: row.get("status"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        contact_id: row.get("contact_id"),
        contact_name: row.get("contact_name"),
        competitor: row.get("competitor"),
        lost_reason: row.get("lost_reason"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_opportunity_line(row: &sqlx::postgres::PgRow) -> OpportunityLine {
    OpportunityLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        opportunity_id: row.get("opportunity_id"),
        line_number: row.get("line_number"),
        product_name: row.get("product_name"),
        product_code: row.get("product_code"),
        description: row.get("description"),
        quantity: get_num(row, "quantity"),
        unit_price: get_num(row, "unit_price"),
        line_amount: get_num(row, "line_amount"),
        discount_percent: get_num(row, "discount_percent"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_activity(row: &sqlx::postgres::PgRow) -> SalesActivity {
    SalesActivity {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        subject: row.get("subject"),
        description: row.get("description"),
        activity_type: row.get("activity_type"),
        status: row.get("status"),
        priority: row.get("priority"),
        lead_id: row.get("lead_id"),
        opportunity_id: row.get("opportunity_id"),
        contact_id: row.get("contact_id"),
        contact_name: row.get("contact_name"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        start_at: row.get("start_at"),
        end_at: row.get("end_at"),
        completed_at: row.get("completed_at"),
        outcome: row.get("outcome"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_stage_history(row: &sqlx::postgres::PgRow) -> OpportunityStageHistory {
    OpportunityStageHistory {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        opportunity_id: row.get("opportunity_id"),
        from_stage: row.get("from_stage"),
        to_stage: row.get("to_stage"),
        changed_by: row.get("changed_by"),
        changed_by_name: row.get("changed_by_name"),
        changed_at: row.get("changed_at"),
        notes: row.get("notes"),
    }
}
