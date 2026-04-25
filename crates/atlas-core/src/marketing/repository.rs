//! Marketing Campaign Repository
//!
//! PostgreSQL storage for marketing campaign data.

use atlas_shared::{
    CampaignType, MarketingCampaign, CampaignMember, CampaignResponse,
    MarketingDashboard, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for marketing campaign data storage
#[async_trait]
pub trait MarketingRepository: Send + Sync {
    // Campaign Types
    async fn create_campaign_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        channel: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CampaignType>;
    async fn get_campaign_type_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CampaignType>>;
    async fn list_campaign_types(&self, org_id: Uuid) -> AtlasResult<Vec<CampaignType>>;
    async fn delete_campaign_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Marketing Campaigns
    async fn create_campaign(
        &self,
        org_id: Uuid,
        campaign_number: &str,
        name: &str,
        description: Option<&str>,
        campaign_type_id: Option<Uuid>,
        campaign_type_name: Option<&str>,
        channel: &str,
        budget: &str,
        currency_code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        expected_responses: i32,
        expected_revenue: &str,
        parent_campaign_id: Option<Uuid>,
        parent_campaign_name: Option<&str>,
        tags: serde_json::Value,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MarketingCampaign>;
    async fn get_campaign(&self, id: Uuid) -> AtlasResult<Option<MarketingCampaign>>;
    async fn get_campaign_by_number(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<Option<MarketingCampaign>>;
    async fn list_campaigns(&self, org_id: Uuid, status: Option<&str>, channel: Option<&str>, owner_id: Option<Uuid>) -> AtlasResult<Vec<MarketingCampaign>>;
    async fn update_campaign_status(&self, id: Uuid, status: &str) -> AtlasResult<MarketingCampaign>;
    async fn activate_campaign(&self, id: Uuid) -> AtlasResult<MarketingCampaign>;
    async fn complete_campaign(&self, id: Uuid) -> AtlasResult<MarketingCampaign>;
    async fn cancel_campaign(&self, id: Uuid) -> AtlasResult<MarketingCampaign>;
    async fn update_campaign_actuals(&self, id: Uuid, actual_cost: &str, actual_responses: i32, actual_revenue: &str, converted_leads: i32, converted_opportunities: i32, converted_won: i32) -> AtlasResult<()>;
    async fn delete_campaign(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<()>;

    // Campaign Members
    async fn add_campaign_member(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        lead_id: Option<Uuid>,
        lead_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CampaignMember>;
    async fn get_campaign_member(&self, id: Uuid) -> AtlasResult<Option<CampaignMember>>;
    async fn list_campaign_members(&self, campaign_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CampaignMember>>;
    async fn update_member_status(&self, id: Uuid, status: &str, response: Option<&str>) -> AtlasResult<CampaignMember>;
    async fn delete_campaign_member(&self, id: Uuid) -> AtlasResult<()>;

    // Campaign Responses
    async fn create_response(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        member_id: Option<Uuid>,
        response_type: &str,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        lead_id: Option<Uuid>,
        description: Option<&str>,
        value: &str,
        currency_code: &str,
        source_url: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CampaignResponse>;
    async fn list_responses(&self, campaign_id: Uuid, response_type: Option<&str>) -> AtlasResult<Vec<CampaignResponse>>;
    async fn delete_response(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MarketingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresMarketingRepository {
    pool: PgPool,
}

impl PostgresMarketingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

use sqlx::Row;

fn row_to_campaign_type(row: &sqlx::postgres::PgRow) -> CampaignType {
    CampaignType {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        channel: row.get("channel"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_campaign(row: &sqlx::postgres::PgRow) -> MarketingCampaign {
    MarketingCampaign {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        campaign_number: row.get("campaign_number"),
        name: row.get("name"),
        description: row.get("description"),
        campaign_type_id: row.get("campaign_type_id"),
        campaign_type_name: row.get("campaign_type_name"),
        status: row.get("status"),
        channel: row.get("channel"),
        budget: get_num(row, "budget"),
        actual_cost: get_num(row, "actual_cost"),
        currency_code: row.get("currency_code"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        expected_responses: row.get("expected_responses"),
        expected_revenue: get_num(row, "expected_revenue"),
        actual_responses: row.get("actual_responses"),
        actual_revenue: get_num(row, "actual_revenue"),
        converted_leads: row.get("converted_leads"),
        converted_opportunities: row.get("converted_opportunities"),
        converted_won: row.get("converted_won"),
        parent_campaign_id: row.get("parent_campaign_id"),
        parent_campaign_name: row.get("parent_campaign_name"),
        tags: row.get("tags"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        activated_at: row.get("activated_at"),
        completed_at: row.get("completed_at"),
        cancelled_at: row.get("cancelled_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_member(row: &sqlx::postgres::PgRow) -> CampaignMember {
    CampaignMember {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        campaign_id: row.get("campaign_id"),
        contact_id: row.get("contact_id"),
        contact_name: row.get("contact_name"),
        contact_email: row.get("contact_email"),
        lead_id: row.get("lead_id"),
        lead_number: row.get("lead_number"),
        status: row.get("status"),
        response: row.get("response"),
        responded_at: row.get("responded_at"),
        converted_contact_id: row.get("converted_contact_id"),
        converted_lead_id: row.get("converted_lead_id"),
        converted_opportunity_id: row.get("converted_opportunity_id"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_response(row: &sqlx::postgres::PgRow) -> CampaignResponse {
    CampaignResponse {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        campaign_id: row.get("campaign_id"),
        member_id: row.get("member_id"),
        response_type: row.get("response_type"),
        contact_id: row.get("contact_id"),
        contact_name: row.get("contact_name"),
        contact_email: row.get("contact_email"),
        lead_id: row.get("lead_id"),
        description: row.get("description"),
        value: get_num(row, "value"),
        currency_code: row.get("currency_code"),
        source_url: row.get("source_url"),
        responded_at: row.get("responded_at"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl MarketingRepository for PostgresMarketingRepository {
    // Campaign Types
    async fn create_campaign_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        channel: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CampaignType> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.campaign_types (organization_id, code, name, description, channel, created_by)
               VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(channel).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign_type(&row))
    }

    async fn get_campaign_type_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CampaignType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.campaign_types WHERE organization_id = $1 AND code = $2 AND is_active = true",
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_campaign_type(&r)))
    }

    async fn list_campaign_types(&self, org_id: Uuid) -> AtlasResult<Vec<CampaignType>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.campaign_types WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_campaign_type).collect())
    }

    async fn delete_campaign_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.campaign_types WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Marketing Campaigns
    async fn create_campaign(
        &self,
        org_id: Uuid,
        campaign_number: &str,
        name: &str,
        description: Option<&str>,
        campaign_type_id: Option<Uuid>,
        campaign_type_name: Option<&str>,
        channel: &str,
        budget: &str,
        currency_code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        expected_responses: i32,
        expected_revenue: &str,
        parent_campaign_id: Option<Uuid>,
        parent_campaign_name: Option<&str>,
        tags: serde_json::Value,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MarketingCampaign> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.marketing_campaigns
                (organization_id, campaign_number, name, description,
                 campaign_type_id, campaign_type_name, channel, budget, currency_code,
                 start_date, end_date, owner_id, owner_name,
                 expected_responses, expected_revenue,
                 parent_campaign_id, parent_campaign_name, tags, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
               RETURNING *"#,
        )
        .bind(org_id).bind(campaign_number).bind(name).bind(description)
        .bind(campaign_type_id).bind(campaign_type_name).bind(channel)
        .bind(budget.parse::<f64>().unwrap_or(0.0)).bind(currency_code)
        .bind(start_date).bind(end_date).bind(owner_id).bind(owner_name)
        .bind(expected_responses).bind(expected_revenue.parse::<f64>().unwrap_or(0.0))
        .bind(parent_campaign_id).bind(parent_campaign_name).bind(&tags).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign(&row))
    }

    async fn get_campaign(&self, id: Uuid) -> AtlasResult<Option<MarketingCampaign>> {
        let row = sqlx::query("SELECT * FROM _atlas.marketing_campaigns WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_campaign(&r)))
    }

    async fn get_campaign_by_number(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<Option<MarketingCampaign>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.marketing_campaigns WHERE organization_id = $1 AND campaign_number = $2",
        )
        .bind(org_id).bind(campaign_number).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_campaign(&r)))
    }

    async fn list_campaigns(&self, org_id: Uuid, status: Option<&str>, channel: Option<&str>, owner_id: Option<Uuid>) -> AtlasResult<Vec<MarketingCampaign>> {
        let rows = match (status, channel, owner_id) {
            (Some(s), None, None) => sqlx::query(
                "SELECT * FROM _atlas.marketing_campaigns WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, Some(c), None) => sqlx::query(
                "SELECT * FROM _atlas.marketing_campaigns WHERE organization_id = $1 AND channel = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(c).fetch_all(&self.pool).await,
            (None, None, Some(oid)) => sqlx::query(
                "SELECT * FROM _atlas.marketing_campaigns WHERE organization_id = $1 AND owner_id = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(oid).fetch_all(&self.pool).await,
            (None, None, None) => sqlx::query(
                "SELECT * FROM _atlas.marketing_campaigns WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.marketing_campaigns WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_campaign).collect())
    }

    async fn update_campaign_status(&self, id: Uuid, status: &str) -> AtlasResult<MarketingCampaign> {
        let row = sqlx::query(
            "UPDATE _atlas.marketing_campaigns SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign(&row))
    }

    async fn activate_campaign(&self, id: Uuid) -> AtlasResult<MarketingCampaign> {
        let row = sqlx::query(
            r#"UPDATE _atlas.marketing_campaigns
               SET status = 'active', activated_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign(&row))
    }

    async fn complete_campaign(&self, id: Uuid) -> AtlasResult<MarketingCampaign> {
        let row = sqlx::query(
            r#"UPDATE _atlas.marketing_campaigns
               SET status = 'completed', completed_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign(&row))
    }

    async fn cancel_campaign(&self, id: Uuid) -> AtlasResult<MarketingCampaign> {
        let row = sqlx::query(
            r#"UPDATE _atlas.marketing_campaigns
               SET status = 'cancelled', cancelled_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_campaign(&row))
    }

    async fn update_campaign_actuals(&self, id: Uuid, actual_cost: &str, actual_responses: i32, actual_revenue: &str, converted_leads: i32, converted_opportunities: i32, converted_won: i32) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.marketing_campaigns
               SET actual_cost = $2, actual_responses = $3, actual_revenue = $4,
                   converted_leads = $5, converted_opportunities = $6, converted_won = $7,
                   updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(actual_cost.parse::<f64>().unwrap_or(0.0))
        .bind(actual_responses)
        .bind(actual_revenue.parse::<f64>().unwrap_or(0.0))
        .bind(converted_leads)
        .bind(converted_opportunities)
        .bind(converted_won)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_campaign(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.marketing_campaigns WHERE organization_id = $1 AND campaign_number = $2")
            .bind(org_id).bind(campaign_number).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Campaign Members
    async fn add_campaign_member(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        lead_id: Option<Uuid>,
        lead_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CampaignMember> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.campaign_members
                (organization_id, campaign_id, contact_id, contact_name, contact_email, lead_id, lead_number, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
        )
        .bind(org_id).bind(campaign_id).bind(contact_id).bind(contact_name)
        .bind(contact_email).bind(lead_id).bind(lead_number).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_member(&row))
    }

    async fn get_campaign_member(&self, id: Uuid) -> AtlasResult<Option<CampaignMember>> {
        let row = sqlx::query("SELECT * FROM _atlas.campaign_members WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_member(&r)))
    }

    async fn list_campaign_members(&self, campaign_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CampaignMember>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.campaign_members WHERE campaign_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(campaign_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.campaign_members WHERE campaign_id = $1 ORDER BY created_at DESC",
            ).bind(campaign_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_member).collect())
    }

    async fn update_member_status(&self, id: Uuid, status: &str, response: Option<&str>) -> AtlasResult<CampaignMember> {
        let row = if response.is_some() {
            sqlx::query(
                r#"UPDATE _atlas.campaign_members
                   SET status = $2, response = $3, responded_at = now(), updated_at = now()
                   WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status).bind(response).fetch_one(&self.pool).await
        } else {
            sqlx::query(
                r#"UPDATE _atlas.campaign_members
                   SET status = $2, updated_at = now()
                   WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status).fetch_one(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_member(&row))
    }

    async fn delete_campaign_member(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.campaign_members WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Campaign Responses
    async fn create_response(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        member_id: Option<Uuid>,
        response_type: &str,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        lead_id: Option<Uuid>,
        description: Option<&str>,
        value: &str,
        currency_code: &str,
        source_url: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CampaignResponse> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.campaign_responses
                (organization_id, campaign_id, member_id, response_type,
                 contact_id, contact_name, contact_email, lead_id,
                 description, value, currency_code, source_url, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING *"#,
        )
        .bind(org_id).bind(campaign_id).bind(member_id).bind(response_type)
        .bind(contact_id).bind(contact_name).bind(contact_email).bind(lead_id)
        .bind(description).bind(value.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(source_url).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_response(&row))
    }

    async fn list_responses(&self, campaign_id: Uuid, response_type: Option<&str>) -> AtlasResult<Vec<CampaignResponse>> {
        let rows = if let Some(rt) = response_type {
            sqlx::query(
                "SELECT * FROM _atlas.campaign_responses WHERE campaign_id = $1 AND response_type = $2 ORDER BY responded_at DESC",
            ).bind(campaign_id).bind(rt).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.campaign_responses WHERE campaign_id = $1 ORDER BY responded_at DESC",
            ).bind(campaign_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_response).collect())
    }

    async fn delete_response(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.campaign_responses WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MarketingDashboard> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COALESCE(SUM(budget), 0) as total_budget,
                COALESCE(SUM(actual_cost), 0) as total_cost,
                COALESCE(SUM(expected_revenue), 0) as expected_rev,
                COALESCE(SUM(actual_revenue), 0) as actual_rev,
                COALESCE(SUM(actual_responses), 0) as total_responses,
                COALESCE(SUM(converted_leads), 0) as total_leads
               FROM _atlas.marketing_campaigns WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let total_cost: f64 = row.try_get("total_cost").unwrap_or(0.0);
        let actual_rev: f64 = row.try_get("actual_rev").unwrap_or(0.0);
        let roi = if total_cost > 0.0 {
            ((actual_rev - total_cost) / total_cost) * 100.0
        } else {
            0.0
        };

        // By status
        let status_rows = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt, COALESCE(SUM(budget), 0) as total_budget
               FROM _atlas.marketing_campaigns WHERE organization_id = $1
               GROUP BY status ORDER BY status"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let by_status: serde_json::Value = status_rows.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
                "budget": format!("{:.2}", r.get::<f64, _>("total_budget")),
            })
        }).collect();

        // By channel
        let channel_rows = sqlx::query(
            r#"SELECT channel, COUNT(*) as cnt, COALESCE(SUM(budget), 0) as total_budget
               FROM _atlas.marketing_campaigns WHERE organization_id = $1
               GROUP BY channel ORDER BY total_budget DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let by_channel: serde_json::Value = channel_rows.iter().map(|r| {
            serde_json::json!({
                "channel": r.get::<String, _>("channel"),
                "count": r.get::<i64, _>("cnt"),
                "budget": format!("{:.2}", r.get::<f64, _>("total_budget")),
            })
        }).collect();

        // Top campaigns by actual_revenue
        let top_rows = sqlx::query(
            r#"SELECT campaign_number, name, actual_revenue as actual_revenue, actual_cost as actual_cost, actual_responses
               FROM _atlas.marketing_campaigns WHERE organization_id = $1 AND status IN ('active', 'completed')
               ORDER BY actual_revenue DESC LIMIT 5"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let top_campaigns: serde_json::Value = top_rows.iter().map(|r| {
            serde_json::json!({
                "campaignNumber": r.get::<String, _>("campaign_number"),
                "name": r.get::<String, _>("name"),
                "actualRevenue": format!("{:.2}", r.get::<f64, _>("actual_revenue")),
                "actualCost": format!("{:.2}", r.get::<f64, _>("actual_cost")),
                "responses": r.get::<i32, _>("actual_responses"),
            })
        }).collect();

        Ok(MarketingDashboard {
            total_campaigns: row.get::<i64, _>("total") as i32,
            active_campaigns: row.get::<i64, _>("active") as i32,
            completed_campaigns: row.get::<i64, _>("completed") as i32,
            total_budget: format!("{:.2}", row.try_get::<f64, _>("total_budget").unwrap_or(0.0)),
            total_actual_cost: format!("{:.2}", total_cost),
            total_expected_revenue: format!("{:.2}", row.try_get::<f64, _>("expected_rev").unwrap_or(0.0)),
            total_actual_revenue: format!("{:.2}", actual_rev),
            total_responses: row.get::<i64, _>("total_responses") as i32,
            total_converted_leads: row.get::<i64, _>("total_leads") as i32,
            overall_roi: format!("{:.1}", roi),
            campaigns_by_status: by_status,
            campaigns_by_channel: by_channel,
            top_campaigns,
        })
    }
}
