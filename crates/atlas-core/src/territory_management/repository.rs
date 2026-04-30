//! Territory Management Repository
//!
//! PostgreSQL storage for territory data.

use atlas_shared::{
    Territory, TerritoryMember, TerritoryRule, TerritoryQuota, TerritoryDashboard,
    AtlasResult, AtlasError,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for territory data storage
#[async_trait]
pub trait TerritoryManagementRepository: Send + Sync {
    // Territory CRUD
    async fn create_territory(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        territory_type: &str,
        parent_id: Option<Uuid>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Territory>;
    async fn get_territory(&self, id: Uuid) -> AtlasResult<Option<Territory>>;
    async fn get_territory_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Territory>>;
    async fn list_territories(
        &self,
        org_id: Uuid,
        territory_type: Option<&str>,
        parent_id: Option<Uuid>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<Territory>>;
    async fn update_territory(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        territory_type: Option<&str>,
        parent_id: Option<Option<Uuid>>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Territory>;
    async fn update_territory_status(&self, id: Uuid, is_active: bool) -> AtlasResult<Territory>;
    async fn delete_territory(&self, id: Uuid) -> AtlasResult<()>;

    // Members
    async fn add_member(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        user_id: Uuid,
        user_name: &str,
        role: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TerritoryMember>;
    async fn find_member(&self, territory_id: Uuid, user_id: Uuid, role: &str) -> AtlasResult<Option<TerritoryMember>>;
    async fn list_members(&self, territory_id: Uuid, role: Option<&str>) -> AtlasResult<Vec<TerritoryMember>>;
    async fn remove_member(&self, member_id: Uuid) -> AtlasResult<()>;

    // Routing Rules
    async fn add_rule(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        entity_type: &str,
        field_name: &str,
        match_operator: &str,
        match_value: &str,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TerritoryRule>;
    async fn list_rules(&self, territory_id: Uuid, entity_type: Option<&str>) -> AtlasResult<Vec<TerritoryRule>>;
    async fn remove_rule(&self, rule_id: Uuid) -> AtlasResult<()>;

    // Quotas
    async fn create_quota(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        period_name: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        revenue_quota: &str,
        actual_revenue: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TerritoryQuota>;
    async fn get_quota(&self, id: Uuid) -> AtlasResult<Option<TerritoryQuota>>;
    async fn find_quota(&self, territory_id: Uuid, period_name: &str) -> AtlasResult<Option<TerritoryQuota>>;
    async fn list_quotas(&self, territory_id: Uuid) -> AtlasResult<Vec<TerritoryQuota>>;
    async fn update_quota_amount(&self, id: Uuid, revenue_quota: &str) -> AtlasResult<TerritoryQuota>;
    async fn update_quota_actual(&self, id: Uuid, actual_revenue: &str) -> AtlasResult<TerritoryQuota>;
    async fn delete_quota(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TerritoryDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresTerritoryManagementRepository {
    pool: PgPool,
}

impl PostgresTerritoryManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TerritoryManagementRepository for PostgresTerritoryManagementRepository {
    async fn create_territory(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        territory_type: &str,
        parent_id: Option<Uuid>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Territory> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.territories
                (organization_id, code, name, description, territory_type,
                 parent_id, owner_id, owner_name, effective_from, effective_to, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(territory_type)
        .bind(parent_id).bind(owner_id).bind(owner_name)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_territory(&row))
    }

    async fn get_territory(&self, id: Uuid) -> AtlasResult<Option<Territory>> {
        let row = sqlx::query("SELECT * FROM _atlas.territories WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_territory(&r)))
    }

    async fn get_territory_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Territory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_territory(&r)))
    }

    async fn list_territories(
        &self,
        org_id: Uuid,
        territory_type: Option<&str>,
        parent_id: Option<Uuid>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<Territory>> {
        let _active_filter = if include_inactive { "" } else { " AND is_active = true" };
        let _type_filter = if territory_type.is_some() { " AND territory_type = $3" } else { "" };
        let _parent_filter = match parent_id {
            Some(_) => " AND parent_id = $4",
            None => "",
        };

        // Simpler approach: build query dynamically
        let rows = match (territory_type, parent_id, include_inactive) {
            (Some(tt), Some(pid), false) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND territory_type = $2 AND parent_id = $3 AND is_active = true ORDER BY name")
                .bind(org_id).bind(tt).bind(pid).fetch_all(&self.pool).await,
            (Some(tt), Some(pid), true) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND territory_type = $2 AND parent_id = $3 ORDER BY name")
                .bind(org_id).bind(tt).bind(pid).fetch_all(&self.pool).await,
            (Some(tt), None, false) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND territory_type = $2 AND is_active = true ORDER BY name")
                .bind(org_id).bind(tt).fetch_all(&self.pool).await,
            (Some(tt), None, true) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND territory_type = $2 ORDER BY name")
                .bind(org_id).bind(tt).fetch_all(&self.pool).await,
            (None, Some(pid), false) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND parent_id = $2 AND is_active = true ORDER BY name")
                .bind(org_id).bind(pid).fetch_all(&self.pool).await,
            (None, Some(pid), true) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND parent_id = $2 ORDER BY name")
                .bind(org_id).bind(pid).fetch_all(&self.pool).await,
            (None, None, false) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 AND is_active = true ORDER BY name")
                .bind(org_id).fetch_all(&self.pool).await,
            (None, None, true) => sqlx::query(
                "SELECT * FROM _atlas.territories WHERE organization_id = $1 ORDER BY name")
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_territory).collect())
    }

    async fn update_territory(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        territory_type: Option<&str>,
        parent_id: Option<Option<Uuid>>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Territory> {
        let row = sqlx::query(
            r#"UPDATE _atlas.territories SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                territory_type = COALESCE($4, territory_type),
                parent_id = COALESCE($5, parent_id),
                owner_id = CASE WHEN $6::boolean THEN $7 ELSE owner_id END,
                owner_name = COALESCE($8, owner_name),
                effective_from = CASE WHEN $9::boolean THEN $10 ELSE effective_from END,
                effective_to = CASE WHEN $11::boolean THEN $12 ELSE effective_to END,
                updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(name).bind(description).bind(territory_type)
        .bind(parent_id)
        .bind(owner_id.is_some()).bind(owner_id).bind(owner_name)
        .bind(effective_from.is_some()).bind(effective_from)
        .bind(effective_to.is_some()).bind(effective_to)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_territory(&row))
    }

    async fn update_territory_status(&self, id: Uuid, is_active: bool) -> AtlasResult<Territory> {
        let row = sqlx::query(
            "UPDATE _atlas.territories SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_territory(&row))
    }

    async fn delete_territory(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.territories WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Members
    async fn add_member(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        user_id: Uuid,
        user_name: &str,
        role: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TerritoryMember> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.territory_members
                (organization_id, territory_id, user_id, user_name, role,
                 effective_from, effective_to, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
               RETURNING *"#,
        )
        .bind(org_id).bind(territory_id).bind(user_id).bind(user_name).bind(role)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_member(&row))
    }

    async fn find_member(&self, territory_id: Uuid, user_id: Uuid, role: &str) -> AtlasResult<Option<TerritoryMember>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.territory_members WHERE territory_id = $1 AND user_id = $2 AND role = $3 AND is_active = true",
        )
        .bind(territory_id).bind(user_id).bind(role)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_member(&r)))
    }

    async fn list_members(&self, territory_id: Uuid, role: Option<&str>) -> AtlasResult<Vec<TerritoryMember>> {
        let rows = match role {
            Some(r) => sqlx::query(
                "SELECT * FROM _atlas.territory_members WHERE territory_id = $1 AND role = $2 AND is_active = true ORDER BY role, user_name")
                .bind(territory_id).bind(r).fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.territory_members WHERE territory_id = $1 AND is_active = true ORDER BY role, user_name")
                .bind(territory_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_member).collect())
    }

    async fn remove_member(&self, member_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.territory_members WHERE id = $1")
            .bind(member_id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Rules
    async fn add_rule(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        entity_type: &str,
        field_name: &str,
        match_operator: &str,
        match_value: &str,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TerritoryRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.territory_rules
                (organization_id, territory_id, entity_type, field_name,
                 match_operator, match_value, priority, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
               RETURNING *"#,
        )
        .bind(org_id).bind(territory_id).bind(entity_type).bind(field_name)
        .bind(match_operator).bind(match_value).bind(priority).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn list_rules(&self, territory_id: Uuid, entity_type: Option<&str>) -> AtlasResult<Vec<TerritoryRule>> {
        let rows = match entity_type {
            Some(et) => sqlx::query(
                "SELECT * FROM _atlas.territory_rules WHERE territory_id = $1 AND entity_type = $2 AND is_active = true ORDER BY priority")
                .bind(territory_id).bind(et).fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.territory_rules WHERE territory_id = $1 AND is_active = true ORDER BY priority")
                .bind(territory_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_rule).collect())
    }

    async fn remove_rule(&self, rule_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.territory_rules WHERE id = $1")
            .bind(rule_id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Quotas
    async fn create_quota(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        period_name: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        revenue_quota: &str,
        actual_revenue: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TerritoryQuota> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.territory_quotas
                (organization_id, territory_id, period_name, period_start, period_end,
                 revenue_quota, actual_revenue, currency_code, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING id, organization_id, territory_id, period_name, period_start, period_end,
                 revenue_quota::text as revenue_quota, actual_revenue::text as actual_revenue,
                 currency_code, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(territory_id).bind(period_name)
        .bind(period_start).bind(period_end)
        .bind(revenue_quota.parse::<f64>().unwrap_or(0.0))
        .bind(actual_revenue.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_quota(&row))
    }

    async fn get_quota(&self, id: Uuid) -> AtlasResult<Option<TerritoryQuota>> {
        let row = sqlx::query("SELECT id, organization_id, territory_id, period_name, period_start, period_end, revenue_quota::text as revenue_quota, actual_revenue::text as actual_revenue, currency_code, created_by, created_at, updated_at FROM _atlas.territory_quotas WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_quota(&r)))
    }

    async fn find_quota(&self, territory_id: Uuid, period_name: &str) -> AtlasResult<Option<TerritoryQuota>> {
        let row = sqlx::query(
            "SELECT id, organization_id, territory_id, period_name, period_start, period_end, revenue_quota::text as revenue_quota, actual_revenue::text as actual_revenue, currency_code, created_by, created_at, updated_at FROM _atlas.territory_quotas WHERE territory_id = $1 AND period_name = $2",
        )
        .bind(territory_id).bind(period_name)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_quota(&r)))
    }

    async fn list_quotas(&self, territory_id: Uuid) -> AtlasResult<Vec<TerritoryQuota>> {
        let rows = sqlx::query(
            "SELECT id, organization_id, territory_id, period_name, period_start, period_end, revenue_quota::text as revenue_quota, actual_revenue::text as actual_revenue, currency_code, created_by, created_at, updated_at FROM _atlas.territory_quotas WHERE territory_id = $1 ORDER BY period_start DESC",
        )
        .bind(territory_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_quota).collect())
    }

    async fn update_quota_amount(&self, id: Uuid, revenue_quota: &str) -> AtlasResult<TerritoryQuota> {
        let row = sqlx::query(
            "UPDATE _atlas.territory_quotas SET revenue_quota = $2::numeric, updated_at = now() WHERE id = $1 RETURNING id, organization_id, territory_id, period_name, period_start, period_end, revenue_quota::text as revenue_quota, actual_revenue::text as actual_revenue, currency_code, created_by, created_at, updated_at",
        )
        .bind(id).bind(revenue_quota.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_quota(&row))
    }

    async fn update_quota_actual(&self, id: Uuid, actual_revenue: &str) -> AtlasResult<TerritoryQuota> {
        let row = sqlx::query(
            "UPDATE _atlas.territory_quotas SET actual_revenue = $2::numeric, updated_at = now() WHERE id = $1 RETURNING id, organization_id, territory_id, period_name, period_start, period_end, revenue_quota::text as revenue_quota, actual_revenue::text as actual_revenue, currency_code, created_by, created_at, updated_at",
        )
        .bind(id).bind(actual_revenue.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_quota(&row))
    }

    async fn delete_quota(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.territory_quotas WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TerritoryDashboard> {
        let terr_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active) as active,
                COUNT(*) FILTER (WHERE parent_id IS NULL) as top_level,
                COUNT(DISTINCT parent_id) FILTER (WHERE parent_id IS NOT NULL) as with_parent
               FROM _atlas.territories WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let member_count: i64 = sqlx::query(
            "SELECT COUNT(DISTINCT user_id) as cnt FROM _atlas.territory_members WHERE organization_id = $1 AND is_active = true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map(|r| r.get("cnt")).unwrap_or(0);

        let quota_row = sqlx::query(
            r#"SELECT
                COALESCE(SUM(revenue_quota), 0) as total_quota,
                COALESCE(SUM(actual_revenue), 0) as total_actual,
                COUNT(*) as quota_count
               FROM _atlas.territory_quotas WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        let total_quota: f64 = quota_row.try_get("total_quota").unwrap_or(0.0);
        let total_actual: f64 = quota_row.try_get("total_actual").unwrap_or(0.0);
        let attainment_pct = if total_quota > 0.0 { (total_actual / total_quota) * 100.0 } else { 0.0 };

        let by_type_row = sqlx::query(
            r#"SELECT territory_type, COUNT(*) as cnt
               FROM _atlas.territories WHERE organization_id = $1 AND is_active = true
               GROUP BY territory_type ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_type: serde_json::Value = by_type_row.iter().map(|r| {
            serde_json::json!({
                "type": r.get::<String, _>("territory_type"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        Ok(TerritoryDashboard {
            total_territories: terr_row.get::<i64, _>("total") as i32,
            active_territories: terr_row.get::<i64, _>("active") as i32,
            top_level_territories: terr_row.get::<i64, _>("top_level") as i32,
            total_members: member_count as i32,
            total_quota: format!("{:.2}", total_quota),
            total_actual: format!("{:.2}", total_actual),
            attainment_percent: format!("{:.1}", attainment_pct),
            quota_count: quota_row.get::<i64, _>("quota_count") as i32,
            by_type,
        })
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

use sqlx::Row;

fn row_to_territory(row: &sqlx::postgres::PgRow) -> Territory {
    Territory {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        territory_type: row.get("territory_type"),
        parent_id: row.get("parent_id"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_member(row: &sqlx::postgres::PgRow) -> TerritoryMember {
    TerritoryMember {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        territory_id: row.get("territory_id"),
        user_id: row.get("user_id"),
        user_name: row.get("user_name"),
        role: row.get("role"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_rule(row: &sqlx::postgres::PgRow) -> TerritoryRule {
    TerritoryRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        territory_id: row.get("territory_id"),
        entity_type: row.get("entity_type"),
        field_name: row.get("field_name"),
        match_operator: row.get("match_operator"),
        match_value: row.get("match_value"),
        priority: row.get("priority"),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_quota(row: &sqlx::postgres::PgRow) -> TerritoryQuota {
    let quota_str: String = row.try_get("revenue_quota").unwrap_or_else(|_| "0".to_string());
    let actual_str: String = row.try_get("actual_revenue").unwrap_or_else(|_| "0".to_string());
    let quota: f64 = quota_str.parse().unwrap_or(0.0);
    let actual: f64 = actual_str.parse().unwrap_or(0.0);
    let pct = if quota > 0.0 { (actual / quota) * 100.0 } else { 0.0 };
    TerritoryQuota {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        territory_id: row.get("territory_id"),
        period_name: row.get("period_name"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        revenue_quota: format!("{:.2}", quota),
        actual_revenue: format!("{:.2}", actual),
        attainment_percent: format!("{:.1}", pct),
        currency_code: row.get("currency_code"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
