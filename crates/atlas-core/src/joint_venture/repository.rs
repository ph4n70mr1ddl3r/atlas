//! Joint Venture Repository
//!
//! PostgreSQL storage for joint ventures, partners, AFEs,
//! cost/revenue distributions, and billings.

use atlas_shared::{
    JointVenture, JointVenturePartner, JointVentureAfe,
    JvCostDistribution, JvCostDistributionLine,
    JvRevenueDistribution, JvRevenueDistributionLine,
    JvBilling, JvBillingLine, JvDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for joint venture data storage
#[async_trait]
pub trait JointVentureRepository: Send + Sync {
    // Joint Ventures
    async fn create_venture(
        &self,
        org_id: Uuid, venture_number: &str, name: &str, description: Option<&str>,
        operator_id: Option<Uuid>, operator_name: Option<&str>,
        currency_code: &str, start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>, accounting_method: &str,
        billing_cycle: &str, cost_cap_amount: Option<&str>,
        cost_cap_currency: Option<&str>,
        gl_revenue_account: Option<&str>, gl_cost_account: Option<&str>,
        gl_billing_account: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<JointVenture>;

    async fn get_venture(&self, id: Uuid) -> AtlasResult<Option<JointVenture>>;
    async fn get_venture_by_number(&self, org_id: Uuid, venture_number: &str) -> AtlasResult<Option<JointVenture>>;
    async fn list_ventures(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JointVenture>>;
    async fn update_venture_status(&self, id: Uuid, status: &str) -> AtlasResult<JointVenture>;

    // Venture Partners
    async fn create_partner(
        &self,
        org_id: Uuid, venture_id: Uuid, partner_id: Uuid, partner_name: &str,
        partner_type: &str, ownership_percentage: &str,
        revenue_interest_pct: Option<&str>, cost_bearing_pct: Option<&str>,
        role: &str, billing_contact: Option<&str>,
        billing_email: Option<&str>, billing_address: Option<&str>,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVenturePartner>;

    async fn get_partner(&self, id: Uuid) -> AtlasResult<Option<JointVenturePartner>>;
    async fn list_partners_by_venture(&self, venture_id: Uuid) -> AtlasResult<Vec<JointVenturePartner>>;
    async fn list_active_partners(&self, venture_id: Uuid, on_date: chrono::NaiveDate) -> AtlasResult<Vec<JointVenturePartner>>;
    async fn update_partner_status(&self, id: Uuid, status: &str) -> AtlasResult<JointVenturePartner>;
    async fn delete_partner(&self, id: Uuid) -> AtlasResult<()>;

    // AFEs
    async fn create_afe(
        &self,
        org_id: Uuid, venture_id: Uuid, afe_number: &str, title: &str,
        description: Option<&str>, estimated_cost: &str, currency_code: &str,
        cost_center: Option<&str>, work_area: Option<&str>, well_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVentureAfe>;

    async fn get_afe(&self, id: Uuid) -> AtlasResult<Option<JointVentureAfe>>;
    async fn get_afe_by_number(&self, org_id: Uuid, afe_number: &str) -> AtlasResult<Option<JointVentureAfe>>;
    async fn list_afes_by_venture(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JointVentureAfe>>;
    async fn update_afe_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<JointVentureAfe>;
    async fn update_afe_costs(
        &self, id: Uuid, actual_cost: &str, committed_cost: &str, remaining_budget: &str,
    ) -> AtlasResult<()>;

    // Cost Distributions
    async fn create_cost_distribution(
        &self,
        org_id: Uuid, venture_id: Uuid, distribution_number: &str,
        afe_id: Option<Uuid>, description: Option<&str>,
        total_amount: &str, currency_code: &str, cost_type: &str,
        distribution_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JvCostDistribution>;

    async fn get_cost_distribution(&self, id: Uuid) -> AtlasResult<Option<JvCostDistribution>>;
    async fn list_cost_distributions(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JvCostDistribution>>;
    async fn update_cost_distribution_status(&self, id: Uuid, status: &str) -> AtlasResult<JvCostDistribution>;

    async fn create_cost_distribution_line(
        &self,
        org_id: Uuid, distribution_id: Uuid, partner_id: Uuid, partner_name: Option<&str>,
        ownership_pct: &str, cost_bearing_pct: &str, distributed_amount: &str,
        gl_account_code: Option<&str>, line_description: Option<&str>,
    ) -> AtlasResult<JvCostDistributionLine>;

    async fn list_cost_distribution_lines(&self, distribution_id: Uuid) -> AtlasResult<Vec<JvCostDistributionLine>>;

    // Revenue Distributions
    async fn create_revenue_distribution(
        &self,
        org_id: Uuid, venture_id: Uuid, distribution_number: &str,
        description: Option<&str>, total_amount: &str, currency_code: &str,
        revenue_type: &str, distribution_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JvRevenueDistribution>;

    async fn get_revenue_distribution(&self, id: Uuid) -> AtlasResult<Option<JvRevenueDistribution>>;
    async fn list_revenue_distributions(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JvRevenueDistribution>>;
    async fn update_revenue_distribution_status(&self, id: Uuid, status: &str) -> AtlasResult<JvRevenueDistribution>;

    async fn create_revenue_distribution_line(
        &self,
        org_id: Uuid, distribution_id: Uuid, partner_id: Uuid, partner_name: Option<&str>,
        revenue_interest_pct: &str, distributed_amount: &str,
        gl_account_code: Option<&str>, line_description: Option<&str>,
    ) -> AtlasResult<JvRevenueDistributionLine>;

    async fn list_revenue_distribution_lines(&self, distribution_id: Uuid) -> AtlasResult<Vec<JvRevenueDistributionLine>>;

    // Billings
    async fn create_billing(
        &self,
        org_id: Uuid, venture_id: Uuid, billing_number: &str,
        partner_id: Uuid, partner_name: Option<&str>, billing_type: &str,
        total_amount: &str, tax_amount: &str, total_with_tax: &str,
        currency_code: &str,
        billing_period_start: chrono::NaiveDate, billing_period_end: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<JvBilling>;

    async fn get_billing(&self, id: Uuid) -> AtlasResult<Option<JvBilling>>;
    async fn get_billing_by_number(&self, org_id: Uuid, billing_number: &str) -> AtlasResult<Option<JvBilling>>;
    async fn list_billings(&self, venture_id: Uuid, status: Option<&str>, billing_type: Option<&str>) -> AtlasResult<Vec<JvBilling>>;
    async fn update_billing_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        payment_reference: Option<&str>, dispute_reason: Option<&str>,
    ) -> AtlasResult<JvBilling>;

    async fn create_billing_line(
        &self,
        org_id: Uuid, billing_id: Uuid, line_number: i32,
        cost_distribution_id: Option<Uuid>, revenue_distribution_id: Option<Uuid>,
        description: Option<&str>, cost_type: Option<&str>, amount: &str,
        ownership_pct: Option<&str>,
    ) -> AtlasResult<JvBillingLine>;

    async fn list_billing_lines(&self, billing_id: Uuid) -> AtlasResult<Vec<JvBillingLine>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<JvDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresJointVentureRepository {
    pool: PgPool,
}

impl PostgresJointVentureRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_venture(&self, row: &sqlx::postgres::PgRow) -> JointVenture {
        JointVenture {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            venture_number: row.get("venture_number"),
            name: row.get("name"),
            description: row.get("description"),
            status: row.get("status"),
            operator_id: row.get("operator_id"),
            operator_name: row.get("operator_name"),
            currency_code: row.get("currency_code"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            accounting_method: row.get("accounting_method"),
            billing_cycle: row.get("billing_cycle"),
            cost_cap_amount: row.try_get("cost_cap_amount").ok().flatten().map(|v: serde_json::Value| v.to_string()),
            cost_cap_currency: row.get("cost_cap_currency"),
            gl_revenue_account: row.get("gl_revenue_account"),
            gl_cost_account: row.get("gl_cost_account"),
            gl_billing_account: row.get("gl_billing_account"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_partner(&self, row: &sqlx::postgres::PgRow) -> JointVenturePartner {
        let ownership: Option<serde_json::Value> = row.try_get("ownership_percentage").ok().flatten();
        let rev_pct: Option<serde_json::Value> = row.try_get("revenue_interest_pct").ok().flatten();
        let cost_pct: Option<serde_json::Value> = row.try_get("cost_bearing_pct").ok().flatten();
        JointVenturePartner {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            venture_id: row.get("venture_id"),
            partner_id: row.get("partner_id"),
            partner_name: row.get("partner_name"),
            partner_type: row.get("partner_type"),
            ownership_percentage: ownership.map(|v| v.to_string()).unwrap_or_default(),
            revenue_interest_pct: rev_pct.map(|v| v.to_string()),
            cost_bearing_pct: cost_pct.map(|v| v.to_string()),
            role: row.get("role"),
            billing_contact: row.get("billing_contact"),
            billing_email: row.get("billing_email"),
            billing_address: row.get("billing_address"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_afe(&self, row: &sqlx::postgres::PgRow) -> JointVentureAfe {
        let estimated: Option<serde_json::Value> = row.try_get("estimated_cost").ok().flatten();
        let actual: Option<serde_json::Value> = row.try_get("actual_cost").ok().flatten();
        let committed: Option<serde_json::Value> = row.try_get("committed_cost").ok().flatten();
        let remaining: Option<serde_json::Value> = row.try_get("remaining_budget").ok().flatten();
        JointVentureAfe {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            venture_id: row.get("venture_id"),
            afe_number: row.get("afe_number"),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            estimated_cost: estimated.map(|v| v.to_string()).unwrap_or_default(),
            actual_cost: actual.map(|v| v.to_string()).unwrap_or_default(),
            committed_cost: committed.map(|v| v.to_string()).unwrap_or_default(),
            remaining_budget: remaining.map(|v| v.to_string()).unwrap_or_default(),
            currency_code: row.get("currency_code"),
            cost_center: row.get("cost_center"),
            work_area: row.get("work_area"),
            well_name: row.get("well_name"),
            requested_by: row.get("requested_by"),
            requested_at: row.get("requested_at"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_cost_dist(&self, row: &sqlx::postgres::PgRow) -> JvCostDistribution {
        let total: Option<serde_json::Value> = row.try_get("total_amount").ok().flatten();
        JvCostDistribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            venture_id: row.get("venture_id"),
            distribution_number: row.get("distribution_number"),
            afe_id: row.get("afe_id"),
            description: row.get("description"),
            status: row.get("status"),
            total_amount: total.map(|v| v.to_string()).unwrap_or_default(),
            currency_code: row.get("currency_code"),
            cost_type: row.get("cost_type"),
            distribution_date: row.get("distribution_date"),
            gl_posting_date: row.get("gl_posting_date"),
            gl_posted_at: row.get("gl_posted_at"),
            source_type: row.get("source_type"),
            source_id: row.get("source_id"),
            source_number: row.get("source_number"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_cost_dist_line(&self, row: &sqlx::postgres::PgRow) -> JvCostDistributionLine {
        let ownership: Option<serde_json::Value> = row.try_get("ownership_pct").ok().flatten();
        let bearing: Option<serde_json::Value> = row.try_get("cost_bearing_pct").ok().flatten();
        let amount: Option<serde_json::Value> = row.try_get("distributed_amount").ok().flatten();
        JvCostDistributionLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            distribution_id: row.get("distribution_id"),
            partner_id: row.get("partner_id"),
            partner_name: row.get("partner_name"),
            ownership_pct: ownership.map(|v| v.to_string()).unwrap_or_default(),
            cost_bearing_pct: bearing.map(|v| v.to_string()).unwrap_or_default(),
            distributed_amount: amount.map(|v| v.to_string()).unwrap_or_default(),
            gl_account_code: row.get("gl_account_code"),
            line_description: row.get("line_description"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_rev_dist(&self, row: &sqlx::postgres::PgRow) -> JvRevenueDistribution {
        let total: Option<serde_json::Value> = row.try_get("total_amount").ok().flatten();
        JvRevenueDistribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            venture_id: row.get("venture_id"),
            distribution_number: row.get("distribution_number"),
            description: row.get("description"),
            status: row.get("status"),
            total_amount: total.map(|v| v.to_string()).unwrap_or_default(),
            currency_code: row.get("currency_code"),
            revenue_type: row.get("revenue_type"),
            distribution_date: row.get("distribution_date"),
            gl_posting_date: row.get("gl_posting_date"),
            gl_posted_at: row.get("gl_posted_at"),
            source_type: row.get("source_type"),
            source_id: row.get("source_id"),
            source_number: row.get("source_number"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_rev_dist_line(&self, row: &sqlx::postgres::PgRow) -> JvRevenueDistributionLine {
        let pct: Option<serde_json::Value> = row.try_get("revenue_interest_pct").ok().flatten();
        let amount: Option<serde_json::Value> = row.try_get("distributed_amount").ok().flatten();
        JvRevenueDistributionLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            distribution_id: row.get("distribution_id"),
            partner_id: row.get("partner_id"),
            partner_name: row.get("partner_name"),
            revenue_interest_pct: pct.map(|v| v.to_string()).unwrap_or_default(),
            distributed_amount: amount.map(|v| v.to_string()).unwrap_or_default(),
            gl_account_code: row.get("gl_account_code"),
            line_description: row.get("line_description"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_billing(&self, row: &sqlx::postgres::PgRow) -> JvBilling {
        let total: Option<serde_json::Value> = row.try_get("total_amount").ok().flatten();
        let tax: Option<serde_json::Value> = row.try_get("tax_amount").ok().flatten();
        let total_wt: Option<serde_json::Value> = row.try_get("total_with_tax").ok().flatten();
        JvBilling {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            venture_id: row.get("venture_id"),
            billing_number: row.get("billing_number"),
            partner_id: row.get("partner_id"),
            partner_name: row.get("partner_name"),
            billing_type: row.get("billing_type"),
            status: row.get("status"),
            total_amount: total.map(|v| v.to_string()).unwrap_or_default(),
            tax_amount: tax.map(|v| v.to_string()).unwrap_or_default(),
            total_with_tax: total_wt.map(|v| v.to_string()).unwrap_or_default(),
            currency_code: row.get("currency_code"),
            billing_period_start: row.get("billing_period_start"),
            billing_period_end: row.get("billing_period_end"),
            due_date: row.get("due_date"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            paid_at: row.get("paid_at"),
            payment_reference: row.get("payment_reference"),
            dispute_reason: row.get("dispute_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_billing_line(&self, row: &sqlx::postgres::PgRow) -> JvBillingLine {
        let amount: Option<serde_json::Value> = row.try_get("amount").ok().flatten();
        let pct: Option<serde_json::Value> = row.try_get("ownership_pct").ok().flatten();
        JvBillingLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            billing_id: row.get("billing_id"),
            line_number: row.get("line_number"),
            cost_distribution_id: row.get("cost_distribution_id"),
            revenue_distribution_id: row.get("revenue_distribution_id"),
            description: row.get("description"),
            cost_type: row.get("cost_type"),
            amount: amount.map(|v| v.to_string()).unwrap_or_default(),
            ownership_pct: pct.map(|v| v.to_string()),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl JointVentureRepository for PostgresJointVentureRepository {
    // ========================================================================
    // Joint Ventures
    // ========================================================================

    async fn create_venture(
        &self,
        org_id: Uuid, venture_number: &str, name: &str, description: Option<&str>,
        operator_id: Option<Uuid>, operator_name: Option<&str>,
        currency_code: &str, start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>, accounting_method: &str,
        billing_cycle: &str, cost_cap_amount: Option<&str>,
        cost_cap_currency: Option<&str>,
        gl_revenue_account: Option<&str>, gl_cost_account: Option<&str>,
        gl_billing_account: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<JointVenture> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_ventures
                (organization_id, venture_number, name, description,
                 operator_id, operator_name, currency_code,
                 start_date, end_date, accounting_method, billing_cycle,
                 cost_cap_amount, cost_cap_currency,
                 gl_revenue_account, gl_cost_account, gl_billing_account,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12::numeric, $13, $14, $15, $16, $17)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(venture_number).bind(name).bind(description)
        .bind(operator_id).bind(operator_name).bind(currency_code)
        .bind(start_date).bind(end_date).bind(accounting_method).bind(billing_cycle)
        .bind(cost_cap_amount).bind(cost_cap_currency)
        .bind(gl_revenue_account).bind(gl_cost_account).bind(gl_billing_account)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_venture(&row))
    }

    async fn get_venture(&self, id: Uuid) -> AtlasResult<Option<JointVenture>> {
        let row = sqlx::query("SELECT * FROM _atlas.joint_ventures WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_venture(&r)))
    }

    async fn get_venture_by_number(&self, org_id: Uuid, venture_number: &str) -> AtlasResult<Option<JointVenture>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.joint_ventures WHERE organization_id = $1 AND venture_number = $2"
        )
        .bind(org_id).bind(venture_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_venture(&r)))
    }

    async fn list_ventures(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JointVenture>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.joint_ventures WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.joint_ventures WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_venture(r)).collect())
    }

    async fn update_venture_status(&self, id: Uuid, status: &str) -> AtlasResult<JointVenture> {
        let row = sqlx::query(
            "UPDATE _atlas.joint_ventures SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_venture(&row))
    }

    // ========================================================================
    // Venture Partners
    // ========================================================================

    async fn create_partner(
        &self,
        org_id: Uuid, venture_id: Uuid, partner_id: Uuid, partner_name: &str,
        partner_type: &str, ownership_percentage: &str,
        revenue_interest_pct: Option<&str>, cost_bearing_pct: Option<&str>,
        role: &str, billing_contact: Option<&str>,
        billing_email: Option<&str>, billing_address: Option<&str>,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVenturePartner> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_partners
                (organization_id, venture_id, partner_id, partner_name,
                 partner_type, ownership_percentage,
                 revenue_interest_pct, cost_bearing_pct,
                 role, billing_contact, billing_email, billing_address,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8::numeric,
                    $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(venture_id).bind(partner_id).bind(partner_name)
        .bind(partner_type).bind(ownership_percentage)
        .bind(revenue_interest_pct).bind(cost_bearing_pct)
        .bind(role).bind(billing_contact).bind(billing_email).bind(billing_address)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_partner(&row))
    }

    async fn get_partner(&self, id: Uuid) -> AtlasResult<Option<JointVenturePartner>> {
        let row = sqlx::query("SELECT * FROM _atlas.joint_venture_partners WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_partner(&r)))
    }

    async fn list_partners_by_venture(&self, venture_id: Uuid) -> AtlasResult<Vec<JointVenturePartner>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_partners WHERE venture_id = $1 ORDER BY partner_name"
        )
        .bind(venture_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_partner(r)).collect())
    }

    async fn list_active_partners(&self, venture_id: Uuid, on_date: chrono::NaiveDate) -> AtlasResult<Vec<JointVenturePartner>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.joint_venture_partners
            WHERE venture_id = $1 AND status = 'active'
              AND effective_from <= $2
              AND (effective_to IS NULL OR effective_to >= $2)
            ORDER BY partner_name
            "#,
        )
        .bind(venture_id).bind(on_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_partner(r)).collect())
    }

    async fn update_partner_status(&self, id: Uuid, status: &str) -> AtlasResult<JointVenturePartner> {
        let row = sqlx::query(
            "UPDATE _atlas.joint_venture_partners SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_partner(&row))
    }

    async fn delete_partner(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.joint_venture_partners WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // AFEs
    // ========================================================================

    async fn create_afe(
        &self,
        org_id: Uuid, venture_id: Uuid, afe_number: &str, title: &str,
        description: Option<&str>, estimated_cost: &str, currency_code: &str,
        cost_center: Option<&str>, work_area: Option<&str>, well_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVentureAfe> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_afes
                (organization_id, venture_id, afe_number, title, description,
                 estimated_cost, remaining_budget, currency_code,
                 cost_center, work_area, well_name,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5,
                    $6::numeric, $6::numeric, $7,
                    $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(venture_id).bind(afe_number).bind(title).bind(description)
        .bind(estimated_cost).bind(currency_code)
        .bind(cost_center).bind(work_area).bind(well_name)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_afe(&row))
    }

    async fn get_afe(&self, id: Uuid) -> AtlasResult<Option<JointVentureAfe>> {
        let row = sqlx::query("SELECT * FROM _atlas.joint_venture_afes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_afe(&r)))
    }

    async fn get_afe_by_number(&self, org_id: Uuid, afe_number: &str) -> AtlasResult<Option<JointVentureAfe>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_afes WHERE organization_id = $1 AND afe_number = $2"
        )
        .bind(org_id).bind(afe_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_afe(&r)))
    }

    async fn list_afes_by_venture(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JointVentureAfe>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_afes WHERE venture_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(venture_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_afes WHERE venture_id = $1 ORDER BY created_at DESC"
            )
            .bind(venture_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_afe(r)).collect())
    }

    async fn update_afe_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<JointVentureAfe> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.joint_venture_afes
            SET status = $2, approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                rejected_reason = $4, requested_at = CASE WHEN $2 = 'submitted' THEN now() ELSE requested_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_afe(&row))
    }

    async fn update_afe_costs(
        &self, id: Uuid, actual_cost: &str, committed_cost: &str, remaining_budget: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.joint_venture_afes
            SET actual_cost = $2::numeric, committed_cost = $3::numeric,
                remaining_budget = $4::numeric, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(actual_cost).bind(committed_cost).bind(remaining_budget)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cost Distributions
    // ========================================================================

    async fn create_cost_distribution(
        &self,
        org_id: Uuid, venture_id: Uuid, distribution_number: &str,
        afe_id: Option<Uuid>, description: Option<&str>,
        total_amount: &str, currency_code: &str, cost_type: &str,
        distribution_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JvCostDistribution> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_cost_distributions
                (organization_id, venture_id, distribution_number,
                 afe_id, description, total_amount, currency_code, cost_type,
                 distribution_date, source_type, source_id, source_number, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(venture_id).bind(distribution_number)
        .bind(afe_id).bind(description).bind(total_amount).bind(currency_code).bind(cost_type)
        .bind(distribution_date).bind(source_type).bind(source_id).bind(source_number).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_cost_dist(&row))
    }

    async fn get_cost_distribution(&self, id: Uuid) -> AtlasResult<Option<JvCostDistribution>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_cost_distributions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_cost_dist(&r)))
    }

    async fn list_cost_distributions(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JvCostDistribution>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_cost_distributions WHERE venture_id = $1 AND status = $2 ORDER BY distribution_date DESC"
            )
            .bind(venture_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_cost_distributions WHERE venture_id = $1 ORDER BY distribution_date DESC"
            )
            .bind(venture_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_cost_dist(r)).collect())
    }

    async fn update_cost_distribution_status(&self, id: Uuid, status: &str) -> AtlasResult<JvCostDistribution> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.joint_venture_cost_distributions
            SET status = $2,
                gl_posted_at = CASE WHEN $2 = 'posted' THEN now() ELSE gl_posted_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_cost_dist(&row))
    }

    async fn create_cost_distribution_line(
        &self,
        org_id: Uuid, distribution_id: Uuid, partner_id: Uuid, partner_name: Option<&str>,
        ownership_pct: &str, cost_bearing_pct: &str, distributed_amount: &str,
        gl_account_code: Option<&str>, line_description: Option<&str>,
    ) -> AtlasResult<JvCostDistributionLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_cost_distribution_lines
                (organization_id, distribution_id, partner_id, partner_name,
                 ownership_pct, cost_bearing_pct, distributed_amount,
                 gl_account_code, line_description)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(distribution_id).bind(partner_id).bind(partner_name)
        .bind(ownership_pct).bind(cost_bearing_pct).bind(distributed_amount)
        .bind(gl_account_code).bind(line_description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_cost_dist_line(&row))
    }

    async fn list_cost_distribution_lines(&self, distribution_id: Uuid) -> AtlasResult<Vec<JvCostDistributionLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_cost_distribution_lines WHERE distribution_id = $1 ORDER BY partner_name"
        )
        .bind(distribution_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_cost_dist_line(r)).collect())
    }

    // ========================================================================
    // Revenue Distributions
    // ========================================================================

    async fn create_revenue_distribution(
        &self,
        org_id: Uuid, venture_id: Uuid, distribution_number: &str,
        description: Option<&str>, total_amount: &str, currency_code: &str,
        revenue_type: &str, distribution_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JvRevenueDistribution> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_revenue_distributions
                (organization_id, venture_id, distribution_number,
                 description, total_amount, currency_code, revenue_type,
                 distribution_date, source_type, source_id, source_number, created_by)
            VALUES ($1, $2, $3, $4, $5::numeric, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(venture_id).bind(distribution_number)
        .bind(description).bind(total_amount).bind(currency_code).bind(revenue_type)
        .bind(distribution_date).bind(source_type).bind(source_id).bind(source_number).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_rev_dist(&row))
    }

    async fn get_revenue_distribution(&self, id: Uuid) -> AtlasResult<Option<JvRevenueDistribution>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_revenue_distributions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_rev_dist(&r)))
    }

    async fn list_revenue_distributions(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JvRevenueDistribution>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_revenue_distributions WHERE venture_id = $1 AND status = $2 ORDER BY distribution_date DESC"
            )
            .bind(venture_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_revenue_distributions WHERE venture_id = $1 ORDER BY distribution_date DESC"
            )
            .bind(venture_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_rev_dist(r)).collect())
    }

    async fn update_revenue_distribution_status(&self, id: Uuid, status: &str) -> AtlasResult<JvRevenueDistribution> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.joint_venture_revenue_distributions
            SET status = $2,
                gl_posted_at = CASE WHEN $2 = 'posted' THEN now() ELSE gl_posted_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_rev_dist(&row))
    }

    async fn create_revenue_distribution_line(
        &self,
        org_id: Uuid, distribution_id: Uuid, partner_id: Uuid, partner_name: Option<&str>,
        revenue_interest_pct: &str, distributed_amount: &str,
        gl_account_code: Option<&str>, line_description: Option<&str>,
    ) -> AtlasResult<JvRevenueDistributionLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_revenue_distribution_lines
                (organization_id, distribution_id, partner_id, partner_name,
                 revenue_interest_pct, distributed_amount,
                 gl_account_code, line_description)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(distribution_id).bind(partner_id).bind(partner_name)
        .bind(revenue_interest_pct).bind(distributed_amount)
        .bind(gl_account_code).bind(line_description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_rev_dist_line(&row))
    }

    async fn list_revenue_distribution_lines(&self, distribution_id: Uuid) -> AtlasResult<Vec<JvRevenueDistributionLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_revenue_distribution_lines WHERE distribution_id = $1 ORDER BY partner_name"
        )
        .bind(distribution_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_rev_dist_line(r)).collect())
    }

    // ========================================================================
    // Billings
    // ========================================================================

    async fn create_billing(
        &self,
        org_id: Uuid, venture_id: Uuid, billing_number: &str,
        partner_id: Uuid, partner_name: Option<&str>, billing_type: &str,
        total_amount: &str, tax_amount: &str, total_with_tax: &str,
        currency_code: &str,
        billing_period_start: chrono::NaiveDate, billing_period_end: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<JvBilling> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_billings
                (organization_id, venture_id, billing_number,
                 partner_id, partner_name, billing_type,
                 total_amount, tax_amount, total_with_tax,
                 currency_code, billing_period_start, billing_period_end,
                 due_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6,
                    $7::numeric, $8::numeric, $9::numeric,
                    $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(venture_id).bind(billing_number)
        .bind(partner_id).bind(partner_name).bind(billing_type)
        .bind(total_amount).bind(tax_amount).bind(total_with_tax)
        .bind(currency_code).bind(billing_period_start).bind(billing_period_end)
        .bind(due_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_billing(&row))
    }

    async fn get_billing(&self, id: Uuid) -> AtlasResult<Option<JvBilling>> {
        let row = sqlx::query("SELECT * FROM _atlas.joint_venture_billings WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_billing(&r)))
    }

    async fn get_billing_by_number(&self, org_id: Uuid, billing_number: &str) -> AtlasResult<Option<JvBilling>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_billings WHERE organization_id = $1 AND billing_number = $2"
        )
        .bind(org_id).bind(billing_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_billing(&r)))
    }

    async fn list_billings(&self, venture_id: Uuid, status: Option<&str>, billing_type: Option<&str>) -> AtlasResult<Vec<JvBilling>> {
        let rows = match (status, billing_type) {
            (Some(s), Some(bt)) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_billings WHERE venture_id = $1 AND status = $2 AND billing_type = $3 ORDER BY created_at DESC"
            )
            .bind(venture_id).bind(s).bind(bt)
            .fetch_all(&self.pool).await,
            (Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_billings WHERE venture_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(venture_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, Some(bt)) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_billings WHERE venture_id = $1 AND billing_type = $2 ORDER BY created_at DESC"
            )
            .bind(venture_id).bind(bt)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.joint_venture_billings WHERE venture_id = $1 ORDER BY created_at DESC"
            )
            .bind(venture_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_billing(r)).collect())
    }

    async fn update_billing_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        payment_reference: Option<&str>, dispute_reason: Option<&str>,
    ) -> AtlasResult<JvBilling> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.joint_venture_billings
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                paid_at = CASE WHEN $2 = 'paid' THEN now() ELSE paid_at END,
                payment_reference = COALESCE($4, payment_reference),
                dispute_reason = $5,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(payment_reference).bind(dispute_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_billing(&row))
    }

    async fn create_billing_line(
        &self,
        org_id: Uuid, billing_id: Uuid, line_number: i32,
        cost_distribution_id: Option<Uuid>, revenue_distribution_id: Option<Uuid>,
        description: Option<&str>, cost_type: Option<&str>, amount: &str,
        ownership_pct: Option<&str>,
    ) -> AtlasResult<JvBillingLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.joint_venture_billing_lines
                (organization_id, billing_id, line_number,
                 cost_distribution_id, revenue_distribution_id,
                 description, cost_type, amount, ownership_pct)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(billing_id).bind(line_number)
        .bind(cost_distribution_id).bind(revenue_distribution_id)
        .bind(description).bind(cost_type).bind(amount).bind(ownership_pct)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_billing_line(&row))
    }

    async fn list_billing_lines(&self, billing_id: Uuid) -> AtlasResult<Vec<JvBillingLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.joint_venture_billing_lines WHERE billing_id = $1 ORDER BY line_number"
        )
        .bind(billing_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_billing_line(r)).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<JvDashboard> {
        // Aggregate from joint_ventures table
        let venture_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_ventures,
                COUNT(*) FILTER (WHERE status = 'active') as active_ventures
            FROM _atlas.joint_ventures
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let partner_row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT partner_id) as total_partners
            FROM _atlas.joint_venture_partners
            WHERE organization_id = $1 AND status = 'active'
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let cost_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(total_amount), 0) as total_cost
            FROM _atlas.joint_venture_cost_distributions
            WHERE organization_id = $1 AND status = 'posted'
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let rev_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(total_amount), 0) as total_rev
            FROM _atlas.joint_venture_revenue_distributions
            WHERE organization_id = $1 AND status = 'posted'
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let billing_row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(total_with_tax) FILTER (WHERE status IN ('submitted', 'approved')), 0) as total_billed,
                COALESCE(SUM(total_with_tax) FILTER (WHERE status = 'paid'), 0) as total_collected
            FROM _atlas.joint_venture_billings
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let afe_row = sqlx::query(
            r#"
            SELECT COUNT(*) as pending_afes
            FROM _atlas.joint_venture_afes
            WHERE organization_id = $1 AND status = 'submitted'
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let status_row = sqlx::query(
            r#"
            SELECT json_object_agg(status, cnt) as by_status
            FROM (
                SELECT status, COUNT(*) as cnt
                FROM _atlas.joint_ventures
                WHERE organization_id = $1
                GROUP BY status
            ) sub
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_ventures: i64 = venture_row.try_get("total_ventures").unwrap_or(0);
        let active_ventures: i64 = venture_row.try_get("active_ventures").unwrap_or(0);
        let total_partners: i64 = partner_row.try_get("total_partners").unwrap_or(0);
        let total_cost: serde_json::Value = cost_row.try_get("total_cost").unwrap_or(serde_json::json!(0));
        let total_rev: serde_json::Value = rev_row.try_get("total_rev").unwrap_or(serde_json::json!(0));
        let total_billed: serde_json::Value = billing_row.try_get("total_billed").unwrap_or(serde_json::json!(0));
        let total_collected: serde_json::Value = billing_row.try_get("total_collected").unwrap_or(serde_json::json!(0));
        let pending_afes: i64 = afe_row.try_get("pending_afes").unwrap_or(0);
        let by_status: serde_json::Value = status_row
            .and_then(|r| r.try_get("by_status").ok())
            .flatten()
            .unwrap_or(serde_json::json!({}));

        let billed: f64 = total_billed.as_f64().unwrap_or(0.0);
        let collected: f64 = total_collected.as_f64().unwrap_or(0.0);

        Ok(JvDashboard {
            total_ventures: total_ventures as i32,
            active_ventures: active_ventures as i32,
            total_partners: total_partners as i32,
            total_cost_distributed: total_cost.to_string(),
            total_revenue_distributed: total_rev.to_string(),
            total_billed: total_billed.to_string(),
            total_collected: total_collected.to_string(),
            outstanding_balance: format!("{:.2}", billed - collected),
            pending_afes: pending_afes as i32,
            ventures_by_status: by_status,
        })
    }
}
