//! Sales Commission Repository
//!
//! PostgreSQL storage for sales commission data: reps, plans, tiers,
//! assignments, quotas, transactions, payouts, and payout lines.

use atlas_shared::{
    SalesRepresentative, CommissionPlan, CommissionRateTier, PlanAssignment,
    SalesQuota, CommissionTransaction, CommissionPayout, CommissionPayoutLine,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for sales commission data storage
#[async_trait]
pub trait SalesCommissionRepository: Send + Sync {
    // ── Sales Representatives ──

    async fn create_rep(
        &self,
        org_id: Uuid,
        rep_code: &str,
        employee_id: Option<Uuid>,
        first_name: &str,
        last_name: &str,
        email: Option<&str>,
        territory_code: Option<&str>,
        territory_name: Option<&str>,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        hire_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesRepresentative>;

    async fn get_rep(&self, org_id: Uuid, rep_code: &str) -> AtlasResult<Option<SalesRepresentative>>;
    async fn get_rep_by_id(&self, id: Uuid) -> AtlasResult<Option<SalesRepresentative>>;
    async fn list_reps(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SalesRepresentative>>;
    async fn delete_rep(&self, org_id: Uuid, rep_code: &str) -> AtlasResult<()>;

    // ── Commission Plans ──

    async fn create_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        basis: &str,
        calculation_method: &str,
        default_rate: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionPlan>;

    async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CommissionPlan>>;
    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<CommissionPlan>>;
    async fn list_plans(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CommissionPlan>>;
    async fn update_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<CommissionPlan>;
    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Commission Rate Tiers ──

    async fn create_rate_tier(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        tier_number: i32,
        from_amount: &str,
        to_amount: Option<&str>,
        rate_percent: &str,
        flat_amount: Option<&str>,
    ) -> AtlasResult<CommissionRateTier>;

    async fn list_rate_tiers(&self, plan_id: Uuid) -> AtlasResult<Vec<CommissionRateTier>>;

    // ── Plan Assignments ──

    async fn create_assignment(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Uuid,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanAssignment>;

    async fn list_assignments(&self, org_id: Uuid, rep_id: Option<Uuid>) -> AtlasResult<Vec<PlanAssignment>>;

    // ── Sales Quotas ──

    async fn create_quota(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Option<Uuid>,
        quota_number: &str,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        quota_type: &str,
        target_amount: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesQuota>;

    async fn get_quota(&self, id: Uuid) -> AtlasResult<Option<SalesQuota>>;
    async fn list_quotas(&self, org_id: Uuid, rep_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SalesQuota>>;
    async fn update_quota_achievement(&self, id: Uuid, achieved: &str, percent: &str) -> AtlasResult<SalesQuota>;

    // ── Commission Transactions ──

    async fn create_transaction(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Option<Uuid>,
        quota_id: Option<Uuid>,
        transaction_number: &str,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        sale_amount: &str,
        commission_basis_amount: &str,
        commission_rate: &str,
        commission_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionTransaction>;

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<CommissionTransaction>>;
    async fn list_transactions(
        &self,
        org_id: Uuid,
        rep_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CommissionTransaction>>;
    async fn update_transaction_status(&self, id: Uuid, status: &str, payout_id: Option<Uuid>) -> AtlasResult<CommissionTransaction>;

    // ── Payouts ──

    async fn create_payout(
        &self,
        org_id: Uuid,
        payout_number: &str,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionPayout>;

    async fn get_payout(&self, id: Uuid) -> AtlasResult<Option<CommissionPayout>>;
    async fn list_payouts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CommissionPayout>>;
    async fn update_payout_totals(
        &self,
        id: Uuid,
        total_amount: &str,
        rep_count: i32,
        transaction_count: i32,
    ) -> AtlasResult<()>;
    async fn update_payout_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<CommissionPayout>;

    // ── Payout Lines ──

    async fn create_payout_line(
        &self,
        org_id: Uuid,
        payout_id: Uuid,
        rep_id: Uuid,
        rep_name: &str,
        plan_id: Option<Uuid>,
        plan_code: Option<&str>,
        gross_commission: &str,
        adjustment_amount: &str,
        net_commission: &str,
        currency_code: &str,
        transaction_count: i32,
    ) -> AtlasResult<CommissionPayoutLine>;

    async fn list_payout_lines(&self, payout_id: Uuid) -> AtlasResult<Vec<CommissionPayoutLine>>;
}

/// PostgreSQL implementation
pub struct PostgresSalesCommissionRepository {
    pool: PgPool,
}

impl PostgresSalesCommissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_rep(&self, row: &sqlx::postgres::PgRow) -> SalesRepresentative {
        SalesRepresentative {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rep_code: row.get("rep_code"),
            employee_id: row.get("employee_id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            territory_code: row.get("territory_code"),
            territory_name: row.get("territory_name"),
            manager_id: row.get("manager_id"),
            manager_name: row.get("manager_name"),
            hire_date: row.get("hire_date"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_plan(&self, row: &sqlx::postgres::PgRow) -> CommissionPlan {
        let default_rate: serde_json::Value = row.try_get("default_rate").unwrap_or(serde_json::json!("0"));
        CommissionPlan {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            plan_type: row.get("plan_type"),
            basis: row.get("basis"),
            calculation_method: row.get("calculation_method"),
            default_rate: default_rate.to_string(),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            status: row.get("status"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_tier(&self, row: &sqlx::postgres::PgRow) -> CommissionRateTier {
        let from_amount: serde_json::Value = row.try_get("from_amount").unwrap_or(serde_json::json!("0"));
        let rate_percent: serde_json::Value = row.try_get("rate_percent").unwrap_or(serde_json::json!("0"));
        CommissionRateTier {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            plan_id: row.get("plan_id"),
            tier_number: row.get("tier_number"),
            from_amount: from_amount.to_string(),
            to_amount: row.get("to_amount"),
            rate_percent: rate_percent.to_string(),
            flat_amount: row.get("flat_amount"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_assignment(&self, row: &sqlx::postgres::PgRow) -> PlanAssignment {
        PlanAssignment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rep_id: row.get("rep_id"),
            plan_id: row.get("plan_id"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            status: row.get("status"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_quota(&self, row: &sqlx::postgres::PgRow) -> SalesQuota {
        let target: serde_json::Value = row.try_get("target_amount").unwrap_or(serde_json::json!("0"));
        let achieved: serde_json::Value = row.try_get("achieved_amount").unwrap_or(serde_json::json!("0"));
        let pct: serde_json::Value = row.try_get("achievement_percent").unwrap_or(serde_json::json!("0"));
        SalesQuota {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rep_id: row.get("rep_id"),
            plan_id: row.get("plan_id"),
            quota_number: row.get("quota_number"),
            period_name: row.get("period_name"),
            period_start_date: row.get("period_start_date"),
            period_end_date: row.get("period_end_date"),
            quota_type: row.get("quota_type"),
            target_amount: target.to_string(),
            achieved_amount: achieved.to_string(),
            achievement_percent: pct.to_string(),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_transaction(&self, row: &sqlx::postgres::PgRow) -> CommissionTransaction {
        let sale: serde_json::Value = row.try_get("sale_amount").unwrap_or(serde_json::json!("0"));
        let basis: serde_json::Value = row.try_get("commission_basis_amount").unwrap_or(serde_json::json!("0"));
        let rate: serde_json::Value = row.try_get("commission_rate").unwrap_or(serde_json::json!("0"));
        let amt: serde_json::Value = row.try_get("commission_amount").unwrap_or(serde_json::json!("0"));
        CommissionTransaction {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rep_id: row.get("rep_id"),
            plan_id: row.get("plan_id"),
            quota_id: row.get("quota_id"),
            transaction_number: row.get("transaction_number"),
            source_type: row.get("source_type"),
            source_id: row.get("source_id"),
            source_number: row.get("source_number"),
            transaction_date: row.get("transaction_date"),
            sale_amount: sale.to_string(),
            commission_basis_amount: basis.to_string(),
            commission_rate: rate.to_string(),
            commission_amount: amt.to_string(),
            currency_code: row.get("currency_code"),
            status: row.get("status"),
            payout_id: row.get("payout_id"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_payout(&self, row: &sqlx::postgres::PgRow) -> CommissionPayout {
        let total: serde_json::Value = row.try_get("total_payout_amount").unwrap_or(serde_json::json!("0"));
        CommissionPayout {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            payout_number: row.get("payout_number"),
            period_name: row.get("period_name"),
            period_start_date: row.get("period_start_date"),
            period_end_date: row.get("period_end_date"),
            total_payout_amount: total.to_string(),
            currency_code: row.get("currency_code"),
            rep_count: row.get("rep_count"),
            transaction_count: row.get("transaction_count"),
            status: row.get("status"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_payout_line(&self, row: &sqlx::postgres::PgRow) -> CommissionPayoutLine {
        let gross: serde_json::Value = row.try_get("gross_commission").unwrap_or(serde_json::json!("0"));
        let adj: serde_json::Value = row.try_get("adjustment_amount").unwrap_or(serde_json::json!("0"));
        let net: serde_json::Value = row.try_get("net_commission").unwrap_or(serde_json::json!("0"));
        CommissionPayoutLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            payout_id: row.get("payout_id"),
            rep_id: row.get("rep_id"),
            rep_name: row.get("rep_name"),
            plan_id: row.get("plan_id"),
            plan_code: row.get("plan_code"),
            gross_commission: gross.to_string(),
            adjustment_amount: adj.to_string(),
            net_commission: net.to_string(),
            currency_code: row.get("currency_code"),
            transaction_count: row.get("transaction_count"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl SalesCommissionRepository for PostgresSalesCommissionRepository {
    // ── Sales Representatives ──

    async fn create_rep(
        &self,
        org_id: Uuid,
        rep_code: &str,
        employee_id: Option<Uuid>,
        first_name: &str,
        last_name: &str,
        email: Option<&str>,
        territory_code: Option<&str>,
        territory_name: Option<&str>,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        hire_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesRepresentative> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sales_reps
                (organization_id, rep_code, employee_id, first_name, last_name,
                 email, territory_code, territory_name, manager_id, manager_name,
                 hire_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, rep_code) DO UPDATE
                SET first_name = $4, last_name = $5, email = $6,
                    territory_code = $7, territory_name = $8,
                    manager_id = $9, manager_name = $10, hire_date = $11,
                    is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rep_code).bind(employee_id)
        .bind(first_name).bind(last_name).bind(email)
        .bind(territory_code).bind(territory_name)
        .bind(manager_id).bind(manager_name)
        .bind(hire_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_rep(&row))
    }

    async fn get_rep(&self, org_id: Uuid, rep_code: &str) -> AtlasResult<Option<SalesRepresentative>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sales_reps WHERE organization_id = $1 AND rep_code = $2 AND is_active = true"
        )
        .bind(org_id).bind(rep_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_rep(&r)))
    }

    async fn get_rep_by_id(&self, id: Uuid) -> AtlasResult<Option<SalesRepresentative>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_reps WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_rep(&r)))
    }

    async fn list_reps(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SalesRepresentative>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.sales_reps WHERE organization_id = $1 AND is_active = true ORDER BY rep_code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.sales_reps WHERE organization_id = $1 ORDER BY rep_code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_rep(&r)).collect())
    }

    async fn delete_rep(&self, org_id: Uuid, rep_code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.sales_reps SET is_active = false, updated_at = now() WHERE organization_id = $1 AND rep_code = $2"
        )
        .bind(org_id).bind(rep_code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Commission Plans ──

    async fn create_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        basis: &str,
        calculation_method: &str,
        default_rate: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionPlan> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.commission_plans
                (organization_id, code, name, description, plan_type, basis,
                 calculation_method, default_rate, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9, $10, $11)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, plan_type = $5, basis = $6,
                    calculation_method = $7, default_rate = $8::numeric,
                    effective_from = $9, effective_to = $10, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(plan_type).bind(basis).bind(calculation_method)
        .bind(default_rate).bind(effective_from).bind(effective_to)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_plan(&row))
    }

    async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CommissionPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.commission_plans WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_plan(&r)))
    }

    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<CommissionPlan>> {
        let row = sqlx::query("SELECT * FROM _atlas.commission_plans WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_plan(&r)))
    }

    async fn list_plans(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CommissionPlan>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.commission_plans WHERE organization_id = $1 AND status = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.commission_plans WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_plan(&r)).collect())
    }

    async fn update_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<CommissionPlan> {
        let row = sqlx::query(
            "UPDATE _atlas.commission_plans SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_plan(&row))
    }

    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.commission_plans SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Commission Rate Tiers ──

    async fn create_rate_tier(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        tier_number: i32,
        from_amount: &str,
        to_amount: Option<&str>,
        rate_percent: &str,
        flat_amount: Option<&str>,
    ) -> AtlasResult<CommissionRateTier> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.commission_rate_tiers
                (organization_id, plan_id, tier_number, from_amount, to_amount,
                 rate_percent, flat_amount)
            VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, $7::numeric)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(plan_id).bind(tier_number)
        .bind(from_amount).bind(to_amount)
        .bind(rate_percent).bind(flat_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_tier(&row))
    }

    async fn list_rate_tiers(&self, plan_id: Uuid) -> AtlasResult<Vec<CommissionRateTier>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.commission_rate_tiers WHERE plan_id = $1 ORDER BY tier_number"
        )
        .bind(plan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_tier(&r)).collect())
    }

    // ── Plan Assignments ──

    async fn create_assignment(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Uuid,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanAssignment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.plan_assignments
                (organization_id, rep_id, plan_id, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rep_id).bind(plan_id)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_assignment(&row))
    }

    async fn list_assignments(&self, org_id: Uuid, rep_id: Option<Uuid>) -> AtlasResult<Vec<PlanAssignment>> {
        let rows = match rep_id {
            Some(rid) => sqlx::query(
                "SELECT * FROM _atlas.plan_assignments WHERE organization_id = $1 AND rep_id = $2 ORDER BY effective_from DESC"
            )
            .bind(org_id).bind(rid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.plan_assignments WHERE organization_id = $1 ORDER BY effective_from DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_assignment(&r)).collect())
    }

    // ── Sales Quotas ──

    async fn create_quota(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Option<Uuid>,
        quota_number: &str,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        quota_type: &str,
        target_amount: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesQuota> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.sales_quotas
                (organization_id, rep_id, plan_id, quota_number, period_name,
                 period_start_date, period_end_date, quota_type, target_amount, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::numeric, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rep_id).bind(plan_id)
        .bind(quota_number).bind(period_name)
        .bind(period_start_date).bind(period_end_date)
        .bind(quota_type).bind(target_amount).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_quota(&row))
    }

    async fn get_quota(&self, id: Uuid) -> AtlasResult<Option<SalesQuota>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_quotas WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_quota(&r)))
    }

    async fn list_quotas(&self, org_id: Uuid, rep_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SalesQuota>> {
        let rows = match (rep_id, status) {
            (Some(rid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.sales_quotas WHERE organization_id = $1 AND rep_id = $2 AND status = $3 ORDER BY period_start_date DESC"
            )
            .bind(org_id).bind(rid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(rid), None) => sqlx::query(
                "SELECT * FROM _atlas.sales_quotas WHERE organization_id = $1 AND rep_id = $2 ORDER BY period_start_date DESC"
            )
            .bind(org_id).bind(rid)
            .fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.sales_quotas WHERE organization_id = $1 AND status = $2 ORDER BY period_start_date DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.sales_quotas WHERE organization_id = $1 ORDER BY period_start_date DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_quota(&r)).collect())
    }

    async fn update_quota_achievement(&self, id: Uuid, achieved: &str, percent: &str) -> AtlasResult<SalesQuota> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_quotas
               SET achieved_amount = $2::numeric, achievement_percent = $3::numeric, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(achieved).bind(percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_quota(&row))
    }

    // ── Commission Transactions ──

    async fn create_transaction(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Option<Uuid>,
        quota_id: Option<Uuid>,
        transaction_number: &str,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        sale_amount: &str,
        commission_basis_amount: &str,
        commission_rate: &str,
        commission_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionTransaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.commission_transactions
                (organization_id, rep_id, plan_id, quota_id, transaction_number,
                 source_type, source_id, source_number, transaction_date,
                 sale_amount, commission_basis_amount, commission_rate,
                 commission_amount, currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::numeric, $11::numeric, $12::numeric,
                    $13::numeric, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rep_id).bind(plan_id).bind(quota_id)
        .bind(transaction_number)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(transaction_date)
        .bind(sale_amount).bind(commission_basis_amount)
        .bind(commission_rate).bind(commission_amount)
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<CommissionTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.commission_transactions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_transaction(&r)))
    }

    async fn list_transactions(
        &self,
        org_id: Uuid,
        rep_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CommissionTransaction>> {
        let rows = match (rep_id, status) {
            (Some(rid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.commission_transactions WHERE organization_id = $1 AND rep_id = $2 AND status = $3 ORDER BY transaction_date DESC"
            )
            .bind(org_id).bind(rid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(rid), None) => sqlx::query(
                "SELECT * FROM _atlas.commission_transactions WHERE organization_id = $1 AND rep_id = $2 ORDER BY transaction_date DESC"
            )
            .bind(org_id).bind(rid)
            .fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.commission_transactions WHERE organization_id = $1 AND status = $2 ORDER BY transaction_date DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.commission_transactions WHERE organization_id = $1 ORDER BY transaction_date DESC LIMIT 200"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_transaction(&r)).collect())
    }

    async fn update_transaction_status(&self, id: Uuid, status: &str, payout_id: Option<Uuid>) -> AtlasResult<CommissionTransaction> {
        let row = sqlx::query(
            "UPDATE _atlas.commission_transactions SET status = $2, payout_id = $3, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status).bind(payout_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transaction(&row))
    }

    // ── Payouts ──

    async fn create_payout(
        &self,
        org_id: Uuid,
        payout_number: &str,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionPayout> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.commission_payouts
                (organization_id, payout_number, period_name,
                 period_start_date, period_end_date, currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payout_number).bind(period_name)
        .bind(period_start_date).bind(period_end_date)
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_payout(&row))
    }

    async fn get_payout(&self, id: Uuid) -> AtlasResult<Option<CommissionPayout>> {
        let row = sqlx::query("SELECT * FROM _atlas.commission_payouts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_payout(&r)))
    }

    async fn list_payouts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CommissionPayout>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.commission_payouts WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.commission_payouts WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_payout(&r)).collect())
    }

    async fn update_payout_totals(
        &self,
        id: Uuid,
        total_amount: &str,
        rep_count: i32,
        transaction_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.commission_payouts
               SET total_payout_amount = $2::numeric, rep_count = $3,
                   transaction_count = $4, updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id).bind(total_amount).bind(rep_count).bind(transaction_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_payout_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<CommissionPayout> {
        let row = sqlx::query(
            r#"UPDATE _atlas.commission_payouts
               SET status = $2, approved_by = $3, rejected_reason = $4,
                   approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_payout(&row))
    }

    // ── Payout Lines ──

    async fn create_payout_line(
        &self,
        org_id: Uuid,
        payout_id: Uuid,
        rep_id: Uuid,
        rep_name: &str,
        plan_id: Option<Uuid>,
        plan_code: Option<&str>,
        gross_commission: &str,
        adjustment_amount: &str,
        net_commission: &str,
        currency_code: &str,
        transaction_count: i32,
    ) -> AtlasResult<CommissionPayoutLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.commission_payout_lines
                (organization_id, payout_id, rep_id, rep_name, plan_id, plan_code,
                 gross_commission, adjustment_amount, net_commission,
                 currency_code, transaction_count)
            VALUES ($1, $2, $3, $4, $5, $6,
                    $7::numeric, $8::numeric, $9::numeric, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payout_id).bind(rep_id)
        .bind(rep_name).bind(plan_id).bind(plan_code)
        .bind(gross_commission).bind(adjustment_amount)
        .bind(net_commission).bind(currency_code)
        .bind(transaction_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_payout_line(&row))
    }

    async fn list_payout_lines(&self, payout_id: Uuid) -> AtlasResult<Vec<CommissionPayoutLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.commission_payout_lines WHERE payout_id = $1 ORDER BY rep_name"
        )
        .bind(payout_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_payout_line(&r)).collect())
    }
}
