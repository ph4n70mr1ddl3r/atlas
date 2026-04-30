//! Cost Accounting Repository
//!
//! PostgreSQL storage for cost accounting data.

use atlas_shared::{
    CostBook, CostElement, CostProfile, StandardCost,
    CostAdjustment, CostAdjustmentLine, CostVariance,
    CostAccountingDashboard,
    AtlasResult, AtlasError,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use sqlx::Row;

/// Repository trait for cost accounting data storage
#[async_trait]
pub trait CostAccountingRepository: Send + Sync {
    // Cost Books
    async fn create_cost_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        costing_method: &str,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostBook>;
    async fn get_cost_book(&self, id: Uuid) -> AtlasResult<Option<CostBook>>;
    async fn get_cost_book_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CostBook>>;
    async fn list_cost_books(
        &self,
        org_id: Uuid,
        costing_method: Option<&str>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<CostBook>>;
    async fn update_cost_book(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        costing_method: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<CostBook>;
    async fn update_cost_book_status(&self, id: Uuid, status: &str) -> AtlasResult<CostBook>;
    async fn delete_cost_book(&self, id: Uuid) -> AtlasResult<()>;

    // Cost Elements
    async fn create_cost_element(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        element_type: &str,
        cost_book_id: Option<Uuid>,
        default_rate: &str,
        rate_uom: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostElement>;
    async fn get_cost_element(&self, id: Uuid) -> AtlasResult<Option<CostElement>>;
    async fn get_cost_element_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CostElement>>;
    async fn list_cost_elements(
        &self,
        org_id: Uuid,
        element_type: Option<&str>,
        cost_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CostElement>>;
    async fn update_cost_element(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        default_rate: Option<&str>,
    ) -> AtlasResult<CostElement>;
    async fn delete_cost_element(&self, id: Uuid) -> AtlasResult<()>;

    // Cost Profiles
    async fn create_cost_profile(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        cost_book_id: Uuid,
        item_id: Option<Uuid>,
        item_name: Option<&str>,
        cost_type: &str,
        lot_level_costing: bool,
        include_landed_costs: bool,
        overhead_absorption_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostProfile>;
    async fn get_cost_profile(&self, id: Uuid) -> AtlasResult<Option<CostProfile>>;
    async fn get_cost_profile_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CostProfile>>;
    async fn list_cost_profiles(
        &self,
        org_id: Uuid,
        cost_book_id: Option<Uuid>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CostProfile>>;
    async fn delete_cost_profile(&self, id: Uuid) -> AtlasResult<()>;

    // Standard Costs
    async fn create_standard_cost(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        cost_profile_id: Option<Uuid>,
        cost_element_id: Uuid,
        item_id: Uuid,
        item_name: Option<&str>,
        standard_cost: &str,
        currency_code: &str,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<StandardCost>;
    async fn get_standard_cost(&self, id: Uuid) -> AtlasResult<Option<StandardCost>>;
    async fn list_standard_costs(
        &self,
        org_id: Uuid,
        cost_book_id: Option<Uuid>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<StandardCost>>;
    async fn update_standard_cost(&self, id: Uuid, standard_cost: &str) -> AtlasResult<StandardCost>;
    async fn supersede_standard_cost(&self, id: Uuid) -> AtlasResult<StandardCost>;
    async fn delete_standard_cost(&self, id: Uuid) -> AtlasResult<()>;

    // Cost Adjustments
    async fn create_cost_adjustment(
        &self,
        org_id: Uuid,
        adjustment_number: &str,
        cost_book_id: Uuid,
        adjustment_type: &str,
        description: Option<&str>,
        reason: Option<&str>,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostAdjustment>;
    async fn get_cost_adjustment(&self, id: Uuid) -> AtlasResult<Option<CostAdjustment>>;
    async fn get_cost_adjustment_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CostAdjustment>>;
    async fn list_cost_adjustments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        adjustment_type: Option<&str>,
    ) -> AtlasResult<Vec<CostAdjustment>>;
    async fn update_adjustment_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
        total_adjustment_amount: Option<&str>,
    ) -> AtlasResult<CostAdjustment>;
    async fn post_adjustment(
        &self,
        id: Uuid,
        posted_by: Uuid,
        total_amount: &str,
    ) -> AtlasResult<CostAdjustment>;
    async fn delete_cost_adjustment(&self, id: Uuid) -> AtlasResult<()>;

    // Cost Adjustment Lines
    async fn create_adjustment_line(
        &self,
        org_id: Uuid,
        adjustment_id: Uuid,
        line_number: i32,
        item_id: Uuid,
        item_name: Option<&str>,
        cost_element_id: Option<Uuid>,
        old_cost: &str,
        new_cost: &str,
        adjustment_amount: &str,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<CostAdjustmentLine>;
    async fn list_adjustment_lines(&self, adjustment_id: Uuid) -> AtlasResult<Vec<CostAdjustmentLine>>;
    async fn delete_adjustment_line(&self, id: Uuid) -> AtlasResult<()>;

    // Cost Variances
    async fn create_cost_variance(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        variance_type: &str,
        variance_date: chrono::NaiveDate,
        item_id: Uuid,
        item_name: Option<&str>,
        cost_element_id: Option<Uuid>,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        standard_cost: &str,
        actual_cost: &str,
        variance_amount: &str,
        variance_percent: &str,
        quantity: &str,
        currency_code: &str,
        accounting_period: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostVariance>;
    async fn get_cost_variance(&self, id: Uuid) -> AtlasResult<Option<CostVariance>>;
    async fn list_cost_variances(
        &self,
        org_id: Uuid,
        variance_type: Option<&str>,
        item_id: Option<Uuid>,
        cost_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CostVariance>>;
    async fn analyze_variance(&self, id: Uuid, notes: &str) -> AtlasResult<CostVariance>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CostAccountingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresCostAccountingRepository {
    pool: PgPool,
}

impl PostgresCostAccountingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper to convert numeric fields
fn numeric_f64(row: &sqlx::postgres::PgRow, col: &str) -> f64 {
    let s: String = row.try_get(col).unwrap_or_else(|_| "0".to_string());
    s.parse().unwrap_or(0.0)
}

fn fmt2(val: f64) -> String {
    format!("{:.2}", val)
}
fn fmt6(val: f64) -> String {
    format!("{:.6}", val)
}

fn row_to_cost_book(row: &sqlx::postgres::PgRow) -> CostBook {
    CostBook {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        costing_method: row.get("costing_method"),
        currency_code: row.get("currency_code"),
        is_active: row.get("is_active"),
        status: row.get("status"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cost_element(row: &sqlx::postgres::PgRow) -> CostElement {
    CostElement {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        element_type: row.get("element_type"),
        cost_book_id: row.get("cost_book_id"),
        is_active: row.get("is_active"),
        default_rate: fmt6(numeric_f64(row, "default_rate")),
        rate_uom: row.get("rate_uom"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cost_profile(row: &sqlx::postgres::PgRow) -> CostProfile {
    CostProfile {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        cost_book_id: row.get("cost_book_id"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        cost_type: row.get("cost_type"),
        lot_level_costing: row.get("lot_level_costing"),
        include_landed_costs: row.get("include_landed_costs"),
        overhead_absorption_method: row.get("overhead_absorption_method"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_standard_cost(row: &sqlx::postgres::PgRow) -> StandardCost {
    StandardCost {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cost_book_id: row.get("cost_book_id"),
        cost_profile_id: row.get("cost_profile_id"),
        cost_element_id: row.get("cost_element_id"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        standard_cost: fmt6(numeric_f64(row, "standard_cost")),
        currency_code: row.get("currency_code"),
        effective_date: row.get("effective_date"),
        status: row.get("status"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cost_adjustment(row: &sqlx::postgres::PgRow) -> CostAdjustment {
    CostAdjustment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        adjustment_number: row.get("adjustment_number"),
        cost_book_id: row.get("cost_book_id"),
        adjustment_type: row.get("adjustment_type"),
        description: row.get("description"),
        reason: row.get("reason"),
        status: row.get("status"),
        total_adjustment_amount: fmt6(numeric_f64(row, "total_adjustment_amount")),
        currency_code: row.get("currency_code"),
        effective_date: row.get("effective_date"),
        posted_at: row.get("posted_at"),
        posted_by: row.get("posted_by"),
        approved_by: row.get("approved_by"),
        rejected_reason: row.get("rejected_reason"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cost_adjustment_line(row: &sqlx::postgres::PgRow) -> CostAdjustmentLine {
    CostAdjustmentLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        adjustment_id: row.get("adjustment_id"),
        line_number: row.get("line_number"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        cost_element_id: row.get("cost_element_id"),
        old_cost: fmt6(numeric_f64(row, "old_cost")),
        new_cost: fmt6(numeric_f64(row, "new_cost")),
        adjustment_amount: fmt6(numeric_f64(row, "adjustment_amount")),
        currency_code: row.get("currency_code"),
        effective_date: row.get("effective_date"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cost_variance(row: &sqlx::postgres::PgRow) -> CostVariance {
    CostVariance {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cost_book_id: row.get("cost_book_id"),
        variance_type: row.get("variance_type"),
        variance_date: row.get("variance_date"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        cost_element_id: row.get("cost_element_id"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        standard_cost: fmt6(numeric_f64(row, "standard_cost")),
        actual_cost: fmt6(numeric_f64(row, "actual_cost")),
        variance_amount: fmt6(numeric_f64(row, "variance_amount")),
        variance_percent: fmt2(numeric_f64(row, "variance_percent")),
        quantity: fmt6(numeric_f64(row, "quantity")),
        currency_code: row.get("currency_code"),
        accounting_period: row.get("accounting_period"),
        is_analyzed: row.get("is_analyzed"),
        analysis_notes: row.get("analysis_notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl CostAccountingRepository for PostgresCostAccountingRepository {
    // ========================================================================
    // Cost Books
    // ========================================================================

    async fn create_cost_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        costing_method: &str,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostBook> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cost_books
                (organization_id, code, name, description, costing_method,
                 currency_code, effective_from, effective_to, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING id, organization_id, code, name, description, costing_method,
                 currency_code, is_active, status, effective_from, effective_to,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(costing_method)
        .bind(currency_code).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_book(&row))
    }

    async fn get_cost_book(&self, id: Uuid) -> AtlasResult<Option<CostBook>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, costing_method, currency_code, is_active, status, effective_from, effective_to, metadata, created_by, created_at, updated_at FROM _atlas.cost_books WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_book(&r)))
    }

    async fn get_cost_book_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CostBook>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, costing_method, currency_code, is_active, status, effective_from, effective_to, metadata, created_by, created_at, updated_at FROM _atlas.cost_books WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_book(&r)))
    }

    async fn list_cost_books(
        &self,
        org_id: Uuid,
        costing_method: Option<&str>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<CostBook>> {
        let cols = "id, organization_id, code, name, description, costing_method, currency_code, is_active, status, effective_from, effective_to, metadata, created_by, created_at, updated_at";
        let rows = match (costing_method, include_inactive) {
            (Some(cm), false) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_books WHERE organization_id = $1 AND costing_method = $2 AND is_active = true ORDER BY name", cols))
                .bind(org_id).bind(cm).fetch_all(&self.pool).await,
            (Some(cm), true) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_books WHERE organization_id = $1 AND costing_method = $2 ORDER BY name", cols))
                .bind(org_id).bind(cm).fetch_all(&self.pool).await,
            (None, false) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_books WHERE organization_id = $1 AND is_active = true ORDER BY name", cols))
                .bind(org_id).fetch_all(&self.pool).await,
            (None, true) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_books WHERE organization_id = $1 ORDER BY name", cols))
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cost_book).collect())
    }

    async fn update_cost_book(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        costing_method: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<CostBook> {
        let row = sqlx::query(
            r#"UPDATE _atlas.cost_books SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                costing_method = COALESCE($4, costing_method),
                effective_from = CASE WHEN $5::boolean THEN $6 ELSE effective_from END,
                effective_to = CASE WHEN $7::boolean THEN $8 ELSE effective_to END,
                updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, code, name, description, costing_method,
                 currency_code, is_active, status, effective_from, effective_to,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(name).bind(description).bind(costing_method)
        .bind(effective_from.is_some()).bind(effective_from)
        .bind(effective_to.is_some()).bind(effective_to)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_book(&row))
    }

    async fn update_cost_book_status(&self, id: Uuid, status: &str) -> AtlasResult<CostBook> {
        let row = sqlx::query(
            r#"UPDATE _atlas.cost_books SET status = $2, is_active = ($2 = 'active'),
                updated_at = now() WHERE id = $1
               RETURNING id, organization_id, code, name, description, costing_method,
                 currency_code, is_active, status, effective_from, effective_to,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_book(&row))
    }

    async fn delete_cost_book(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cost_books WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cost Elements
    // ========================================================================

    async fn create_cost_element(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        element_type: &str,
        cost_book_id: Option<Uuid>,
        default_rate: &str,
        rate_uom: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostElement> {
        let rate: f64 = default_rate.parse().unwrap_or(0.0);
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cost_elements
                (organization_id, code, name, description, element_type,
                 cost_book_id, default_rate, rate_uom, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING id, organization_id, code, name, description, element_type,
                 cost_book_id, is_active, default_rate::text as default_rate, rate_uom,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(element_type)
        .bind(cost_book_id).bind(rate).bind(rate_uom).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_element(&row))
    }

    async fn get_cost_element(&self, id: Uuid) -> AtlasResult<Option<CostElement>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, element_type, cost_book_id, is_active, default_rate::text as default_rate, rate_uom, metadata, created_by, created_at, updated_at FROM _atlas.cost_elements WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_element(&r)))
    }

    async fn get_cost_element_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CostElement>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, element_type, cost_book_id, is_active, default_rate::text as default_rate, rate_uom, metadata, created_by, created_at, updated_at FROM _atlas.cost_elements WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_element(&r)))
    }

    async fn list_cost_elements(
        &self,
        org_id: Uuid,
        element_type: Option<&str>,
        cost_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CostElement>> {
        let cols = "id, organization_id, code, name, description, element_type, cost_book_id, is_active, default_rate::text as default_rate, rate_uom, metadata, created_by, created_at, updated_at";
        let rows = match (element_type, cost_book_id) {
            (Some(et), Some(cb)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_elements WHERE organization_id = $1 AND element_type = $2 AND cost_book_id = $3 ORDER BY name", cols))
                .bind(org_id).bind(et).bind(cb).fetch_all(&self.pool).await,
            (Some(et), None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_elements WHERE organization_id = $1 AND element_type = $2 ORDER BY name", cols))
                .bind(org_id).bind(et).fetch_all(&self.pool).await,
            (None, Some(cb)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_elements WHERE organization_id = $1 AND cost_book_id = $2 ORDER BY name", cols))
                .bind(org_id).bind(cb).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_elements WHERE organization_id = $1 ORDER BY name", cols))
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cost_element).collect())
    }

    async fn update_cost_element(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        default_rate: Option<&str>,
    ) -> AtlasResult<CostElement> {
        let rate: Option<f64> = default_rate.map(|r| r.parse().unwrap_or(0.0));
        let row = sqlx::query(
            r#"UPDATE _atlas.cost_elements SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                default_rate = COALESCE($4, default_rate),
                updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, code, name, description, element_type,
                 cost_book_id, is_active, default_rate::text as default_rate, rate_uom,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(name).bind(description).bind(rate)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_element(&row))
    }

    async fn delete_cost_element(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cost_elements WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cost Profiles
    // ========================================================================

    async fn create_cost_profile(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        cost_book_id: Uuid,
        item_id: Option<Uuid>,
        item_name: Option<&str>,
        cost_type: &str,
        lot_level_costing: bool,
        include_landed_costs: bool,
        overhead_absorption_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostProfile> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cost_profiles
                (organization_id, code, name, description, cost_book_id,
                 item_id, item_name, cost_type, lot_level_costing,
                 include_landed_costs, overhead_absorption_method, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
               RETURNING id, organization_id, code, name, description, cost_book_id,
                 item_id, item_name, cost_type, lot_level_costing,
                 include_landed_costs, overhead_absorption_method, is_active,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(cost_book_id)
        .bind(item_id).bind(item_name).bind(cost_type).bind(lot_level_costing)
        .bind(include_landed_costs).bind(overhead_absorption_method).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_profile(&row))
    }

    async fn get_cost_profile(&self, id: Uuid) -> AtlasResult<Option<CostProfile>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, cost_book_id, item_id, item_name, cost_type, lot_level_costing, include_landed_costs, overhead_absorption_method, is_active, metadata, created_by, created_at, updated_at FROM _atlas.cost_profiles WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_profile(&r)))
    }

    async fn get_cost_profile_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CostProfile>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, cost_book_id, item_id, item_name, cost_type, lot_level_costing, include_landed_costs, overhead_absorption_method, is_active, metadata, created_by, created_at, updated_at FROM _atlas.cost_profiles WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_profile(&r)))
    }

    async fn list_cost_profiles(
        &self,
        org_id: Uuid,
        cost_book_id: Option<Uuid>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CostProfile>> {
        let cols = "id, organization_id, code, name, description, cost_book_id, item_id, item_name, cost_type, lot_level_costing, include_landed_costs, overhead_absorption_method, is_active, metadata, created_by, created_at, updated_at";
        let rows = match (cost_book_id, item_id) {
            (Some(cb), Some(ii)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_profiles WHERE organization_id = $1 AND cost_book_id = $2 AND item_id = $3 ORDER BY name", cols))
                .bind(org_id).bind(cb).bind(ii).fetch_all(&self.pool).await,
            (Some(cb), None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_profiles WHERE organization_id = $1 AND cost_book_id = $2 ORDER BY name", cols))
                .bind(org_id).bind(cb).fetch_all(&self.pool).await,
            (None, Some(ii)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_profiles WHERE organization_id = $1 AND item_id = $2 ORDER BY name", cols))
                .bind(org_id).bind(ii).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_profiles WHERE organization_id = $1 ORDER BY name", cols))
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cost_profile).collect())
    }

    async fn delete_cost_profile(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cost_profiles WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Standard Costs
    // ========================================================================

    async fn create_standard_cost(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        cost_profile_id: Option<Uuid>,
        cost_element_id: Uuid,
        item_id: Uuid,
        item_name: Option<&str>,
        standard_cost: &str,
        currency_code: &str,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<StandardCost> {
        let cost: f64 = standard_cost.parse().unwrap_or(0.0);
        let row = sqlx::query(
            r#"INSERT INTO _atlas.standard_costs
                (organization_id, cost_book_id, cost_profile_id, cost_element_id,
                 item_id, item_name, standard_cost, currency_code, effective_date, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               RETURNING id, organization_id, cost_book_id, cost_profile_id, cost_element_id,
                 item_id, item_name, standard_cost::text as standard_cost, currency_code,
                 effective_date, status, is_active, metadata, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(cost_book_id).bind(cost_profile_id).bind(cost_element_id)
        .bind(item_id).bind(item_name).bind(cost).bind(currency_code)
        .bind(effective_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_standard_cost(&row))
    }

    async fn get_standard_cost(&self, id: Uuid) -> AtlasResult<Option<StandardCost>> {
        let row = sqlx::query(
            "SELECT id, organization_id, cost_book_id, cost_profile_id, cost_element_id, item_id, item_name, standard_cost::text as standard_cost, currency_code, effective_date, status, is_active, metadata, created_by, created_at, updated_at FROM _atlas.standard_costs WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_standard_cost(&r)))
    }

    async fn list_standard_costs(
        &self,
        org_id: Uuid,
        cost_book_id: Option<Uuid>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<StandardCost>> {
        let cols = "id, organization_id, cost_book_id, cost_profile_id, cost_element_id, item_id, item_name, standard_cost::text as standard_cost, currency_code, effective_date, status, is_active, metadata, created_by, created_at, updated_at";
        let rows = match (cost_book_id, item_id) {
            (Some(cb), Some(ii)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.standard_costs WHERE organization_id = $1 AND cost_book_id = $2 AND item_id = $3 AND status = 'active' ORDER BY effective_date DESC", cols))
                .bind(org_id).bind(cb).bind(ii).fetch_all(&self.pool).await,
            (Some(cb), None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.standard_costs WHERE organization_id = $1 AND cost_book_id = $2 AND status = 'active' ORDER BY effective_date DESC", cols))
                .bind(org_id).bind(cb).fetch_all(&self.pool).await,
            (None, Some(ii)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.standard_costs WHERE organization_id = $1 AND item_id = $2 AND status = 'active' ORDER BY effective_date DESC", cols))
                .bind(org_id).bind(ii).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.standard_costs WHERE organization_id = $1 AND status = 'active' ORDER BY effective_date DESC", cols))
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_standard_cost).collect())
    }

    async fn update_standard_cost(&self, id: Uuid, standard_cost: &str) -> AtlasResult<StandardCost> {
        let cost: f64 = standard_cost.parse().unwrap_or(0.0);
        let row = sqlx::query(
            r#"UPDATE _atlas.standard_costs SET standard_cost = $2, updated_at = now() WHERE id = $1
               RETURNING id, organization_id, cost_book_id, cost_profile_id, cost_element_id,
                 item_id, item_name, standard_cost::text as standard_cost, currency_code,
                 effective_date, status, is_active, metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(cost)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_standard_cost(&row))
    }

    async fn supersede_standard_cost(&self, id: Uuid) -> AtlasResult<StandardCost> {
        let row = sqlx::query(
            r#"UPDATE _atlas.standard_costs SET status = 'superseded', is_active = false, updated_at = now() WHERE id = $1
               RETURNING id, organization_id, cost_book_id, cost_profile_id, cost_element_id,
                 item_id, item_name, standard_cost::text as standard_cost, currency_code,
                 effective_date, status, is_active, metadata, created_by, created_at, updated_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_standard_cost(&row))
    }

    async fn delete_standard_cost(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.standard_costs WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cost Adjustments
    // ========================================================================

    async fn create_cost_adjustment(
        &self,
        org_id: Uuid,
        adjustment_number: &str,
        cost_book_id: Uuid,
        adjustment_type: &str,
        description: Option<&str>,
        reason: Option<&str>,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostAdjustment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cost_adjustments
                (organization_id, adjustment_number, cost_book_id, adjustment_type,
                 description, reason, currency_code, effective_date, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING id, organization_id, adjustment_number, cost_book_id, adjustment_type,
                 description, reason, status,
                 total_adjustment_amount::text as total_adjustment_amount, currency_code,
                 effective_date, posted_at, posted_by, approved_by, rejected_reason,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(adjustment_number).bind(cost_book_id).bind(adjustment_type)
        .bind(description).bind(reason).bind(currency_code).bind(effective_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_adjustment(&row))
    }

    async fn get_cost_adjustment(&self, id: Uuid) -> AtlasResult<Option<CostAdjustment>> {
        let row = sqlx::query(
            "SELECT id, organization_id, adjustment_number, cost_book_id, adjustment_type, description, reason, status, total_adjustment_amount::text as total_adjustment_amount, currency_code, effective_date, posted_at, posted_by, approved_by, rejected_reason, metadata, created_by, created_at, updated_at FROM _atlas.cost_adjustments WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_adjustment(&r)))
    }

    async fn get_cost_adjustment_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CostAdjustment>> {
        let row = sqlx::query(
            "SELECT id, organization_id, adjustment_number, cost_book_id, adjustment_type, description, reason, status, total_adjustment_amount::text as total_adjustment_amount, currency_code, effective_date, posted_at, posted_by, approved_by, rejected_reason, metadata, created_by, created_at, updated_at FROM _atlas.cost_adjustments WHERE organization_id = $1 AND adjustment_number = $2",
        )
        .bind(org_id).bind(number).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_adjustment(&r)))
    }

    async fn list_cost_adjustments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        adjustment_type: Option<&str>,
    ) -> AtlasResult<Vec<CostAdjustment>> {
        let cols = "id, organization_id, adjustment_number, cost_book_id, adjustment_type, description, reason, status, total_adjustment_amount::text as total_adjustment_amount, currency_code, effective_date, posted_at, posted_by, approved_by, rejected_reason, metadata, created_by, created_at, updated_at";
        let rows = match (status, adjustment_type) {
            (Some(s), Some(at)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_adjustments WHERE organization_id = $1 AND status = $2 AND adjustment_type = $3 ORDER BY created_at DESC", cols))
                .bind(org_id).bind(s).bind(at).fetch_all(&self.pool).await,
            (Some(s), None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_adjustments WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC", cols))
                .bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, Some(at)) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_adjustments WHERE organization_id = $1 AND adjustment_type = $2 ORDER BY created_at DESC", cols))
                .bind(org_id).bind(at).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_adjustments WHERE organization_id = $1 ORDER BY created_at DESC", cols))
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cost_adjustment).collect())
    }

    async fn update_adjustment_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
        total_adjustment_amount: Option<&str>,
    ) -> AtlasResult<CostAdjustment> {
        let total: Option<f64> = total_adjustment_amount.map(|v| v.parse().unwrap_or(0.0));
        let row = sqlx::query(
            r#"UPDATE _atlas.cost_adjustments SET
                status = $2,
                approved_by = COALESCE($3, approved_by),
                rejected_reason = COALESCE($4, rejected_reason),
                total_adjustment_amount = COALESCE($5, total_adjustment_amount),
                updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, adjustment_number, cost_book_id, adjustment_type,
                 description, reason, status,
                 total_adjustment_amount::text as total_adjustment_amount, currency_code,
                 effective_date, posted_at, posted_by, approved_by, rejected_reason,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason).bind(total)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_adjustment(&row))
    }

    async fn post_adjustment(
        &self,
        id: Uuid,
        posted_by: Uuid,
        total_amount: &str,
    ) -> AtlasResult<CostAdjustment> {
        let total: f64 = total_amount.parse().unwrap_or(0.0);
        let row = sqlx::query(
            r#"UPDATE _atlas.cost_adjustments SET
                status = 'posted', posted_by = $2, posted_at = now(),
                total_adjustment_amount = $3, updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, adjustment_number, cost_book_id, adjustment_type,
                 description, reason, status,
                 total_adjustment_amount::text as total_adjustment_amount, currency_code,
                 effective_date, posted_at, posted_by, approved_by, rejected_reason,
                 metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(posted_by).bind(total)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_adjustment(&row))
    }

    async fn delete_cost_adjustment(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cost_adjustments WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cost Adjustment Lines
    // ========================================================================

    async fn create_adjustment_line(
        &self,
        org_id: Uuid,
        adjustment_id: Uuid,
        line_number: i32,
        item_id: Uuid,
        item_name: Option<&str>,
        cost_element_id: Option<Uuid>,
        old_cost: &str,
        new_cost: &str,
        adjustment_amount: &str,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<CostAdjustmentLine> {
        let oc: f64 = old_cost.parse().unwrap_or(0.0);
        let nc: f64 = new_cost.parse().unwrap_or(0.0);
        let adj: f64 = adjustment_amount.parse().unwrap_or(0.0);
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cost_adjustment_lines
                (organization_id, adjustment_id, line_number, item_id, item_name,
                 cost_element_id, old_cost, new_cost, adjustment_amount, currency_code, effective_date)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               RETURNING id, organization_id, adjustment_id, line_number, item_id, item_name,
                 cost_element_id, old_cost::text as old_cost, new_cost::text as new_cost,
                 adjustment_amount::text as adjustment_amount, currency_code, effective_date,
                 metadata, created_at, updated_at"#,
        )
        .bind(org_id).bind(adjustment_id).bind(line_number).bind(item_id).bind(item_name)
        .bind(cost_element_id).bind(oc).bind(nc).bind(adj).bind(currency_code).bind(effective_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_adjustment_line(&row))
    }

    async fn list_adjustment_lines(&self, adjustment_id: Uuid) -> AtlasResult<Vec<CostAdjustmentLine>> {
        let rows = sqlx::query(
            "SELECT id, organization_id, adjustment_id, line_number, item_id, item_name, cost_element_id, old_cost::text as old_cost, new_cost::text as new_cost, adjustment_amount::text as adjustment_amount, currency_code, effective_date, metadata, created_at, updated_at FROM _atlas.cost_adjustment_lines WHERE adjustment_id = $1 ORDER BY line_number",
        )
        .bind(adjustment_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cost_adjustment_line).collect())
    }

    async fn delete_adjustment_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cost_adjustment_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cost Variances
    // ========================================================================

    async fn create_cost_variance(
        &self,
        org_id: Uuid,
        cost_book_id: Uuid,
        variance_type: &str,
        variance_date: chrono::NaiveDate,
        item_id: Uuid,
        item_name: Option<&str>,
        cost_element_id: Option<Uuid>,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        standard_cost: &str,
        actual_cost: &str,
        variance_amount: &str,
        variance_percent: &str,
        quantity: &str,
        currency_code: &str,
        accounting_period: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CostVariance> {
        let sc: f64 = standard_cost.parse().unwrap_or(0.0);
        let ac: f64 = actual_cost.parse().unwrap_or(0.0);
        let va: f64 = variance_amount.parse().unwrap_or(0.0);
        let vp: f64 = variance_percent.parse().unwrap_or(0.0);
        let qty: f64 = quantity.parse().unwrap_or(0.0);
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cost_variances
                (organization_id, cost_book_id, variance_type, variance_date,
                 item_id, item_name, cost_element_id, source_type, source_id, source_number,
                 standard_cost, actual_cost, variance_amount, variance_percent, quantity,
                 currency_code, accounting_period, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
               RETURNING id, organization_id, cost_book_id, variance_type, variance_date,
                 item_id, item_name, cost_element_id, source_type, source_id, source_number,
                 standard_cost::text as standard_cost, actual_cost::text as actual_cost,
                 variance_amount::text as variance_amount, variance_percent::text as variance_percent,
                 quantity::text as quantity, currency_code, accounting_period,
                 is_analyzed, analysis_notes, metadata, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(cost_book_id).bind(variance_type).bind(variance_date)
        .bind(item_id).bind(item_name).bind(cost_element_id).bind(source_type)
        .bind(source_id).bind(source_number)
        .bind(sc).bind(ac).bind(va).bind(vp).bind(qty)
        .bind(currency_code).bind(accounting_period).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_variance(&row))
    }

    async fn get_cost_variance(&self, id: Uuid) -> AtlasResult<Option<CostVariance>> {
        let row = sqlx::query(
            "SELECT id, organization_id, cost_book_id, variance_type, variance_date, item_id, item_name, cost_element_id, source_type, source_id, source_number, standard_cost::text as standard_cost, actual_cost::text as actual_cost, variance_amount::text as variance_amount, variance_percent::text as variance_percent, quantity::text as quantity, currency_code, accounting_period, is_analyzed, analysis_notes, metadata, created_by, created_at, updated_at FROM _atlas.cost_variances WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cost_variance(&r)))
    }

    async fn list_cost_variances(
        &self,
        org_id: Uuid,
        variance_type: Option<&str>,
        item_id: Option<Uuid>,
        cost_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<CostVariance>> {
        let cols = "id, organization_id, cost_book_id, variance_type, variance_date, item_id, item_name, cost_element_id, source_type, source_id, source_number, standard_cost::text as standard_cost, actual_cost::text as actual_cost, variance_amount::text as variance_amount, variance_percent::text as variance_percent, quantity::text as quantity, currency_code, accounting_period, is_analyzed, analysis_notes, metadata, created_by, created_at, updated_at";
        // Simplified: filter only when params are provided
        let rows = if variance_type.is_some() || item_id.is_some() || cost_book_id.is_some() {
            let mut query = format!("SELECT {} FROM _atlas.cost_variances WHERE organization_id = $1", cols);
            let mut bind_idx = 2u32;
            let vt = variance_type;
            let ii = item_id;
            let cb = cost_book_id;
            if vt.is_some() { query.push_str(&format!(" AND variance_type = ${bind_idx}")); bind_idx += 1; }
            if ii.is_some() { query.push_str(&format!(" AND item_id = ${bind_idx}")); bind_idx += 1; }
            if cb.is_some() { query.push_str(&format!(" AND cost_book_id = ${bind_idx}")); }
            query.push_str(" ORDER BY variance_date DESC");
            let mut q = sqlx::query(&query).bind(org_id);
            if let Some(v) = vt { q = q.bind(v); }
            if let Some(v) = ii { q = q.bind(v); }
            if let Some(v) = cb { q = q.bind(v); }
            q.fetch_all(&self.pool).await
        } else {
            sqlx::query(&format!(
                "SELECT {} FROM _atlas.cost_variances WHERE organization_id = $1 ORDER BY variance_date DESC", cols))
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cost_variance).collect())
    }

    async fn analyze_variance(&self, id: Uuid, notes: &str) -> AtlasResult<CostVariance> {
        let row = sqlx::query(
            r#"UPDATE _atlas.cost_variances SET is_analyzed = true, analysis_notes = $2, updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, cost_book_id, variance_type, variance_date,
                 item_id, item_name, cost_element_id, source_type, source_id, source_number,
                 standard_cost::text as standard_cost, actual_cost::text as actual_cost,
                 variance_amount::text as variance_amount, variance_percent::text as variance_percent,
                 quantity::text as quantity, currency_code, accounting_period,
                 is_analyzed, analysis_notes, metadata, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cost_variance(&row))
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CostAccountingDashboard> {
        let book_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active = true) as active
               FROM _atlas.cost_books WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let elem_row = sqlx::query(
            "SELECT COUNT(*) as total FROM _atlas.cost_elements WHERE organization_id = $1 AND is_active = true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let sc_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COALESCE(SUM(standard_cost), 0) as total_value
               FROM _atlas.standard_costs WHERE organization_id = $1 AND status = 'active'"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let adj_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('draft', 'submitted')) as pending
               FROM _atlas.cost_adjustments WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let var_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE variance_amount > 0) as unfavorable,
                COALESCE(SUM(variance_amount), 0) as total_variance
               FROM _atlas.cost_variances WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_sc_value: f64 = sc_row.try_get("total_value").unwrap_or(0.0);
        let total_variance: f64 = var_row.try_get("total_variance").unwrap_or(0.0);

        // Breakdown queries
        let by_type = sqlx::query(
            r#"SELECT variance_type, COUNT(*) as cnt, COALESCE(SUM(variance_amount), 0) as total
               FROM _atlas.cost_variances WHERE organization_id = $1 GROUP BY variance_type ORDER BY total DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_element = sqlx::query(
            r#"SELECT ce.element_type, COUNT(*) as cnt, COALESCE(SUM(cv.variance_amount), 0) as total
               FROM _atlas.cost_variances cv
               LEFT JOIN _atlas.cost_elements ce ON cv.cost_element_id = ce.id
               WHERE cv.organization_id = $1
               GROUP BY ce.element_type ORDER BY total DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_status = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.cost_adjustments
               WHERE organization_id = $1 GROUP BY status ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let variance_by_type: serde_json::Value = by_type.iter().map(|r| {
            serde_json::json!({
                "type": r.get::<String, _>("variance_type"),
                "count": r.get::<i64, _>("cnt"),
                "total": format!("{:.2}", r.get::<f64, _>("total")),
            })
        }).collect();

        let variance_by_element: serde_json::Value = by_element.iter().map(|r| {
            serde_json::json!({
                "element_type": r.get::<Option<String>, _>("element_type").unwrap_or("unknown".to_string()),
                "count": r.get::<i64, _>("cnt"),
                "total": format!("{:.2}", r.get::<f64, _>("total")),
            })
        }).collect();

        let adjustments_by_status: serde_json::Value = by_status.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        Ok(CostAccountingDashboard {
            total_cost_books: book_row.get::<i64, _>("total") as i32,
            active_cost_books: book_row.get::<i64, _>("active") as i32,
            total_cost_elements: elem_row.get::<i64, _>("total") as i32,
            total_standard_costs: sc_row.get::<i64, _>("total") as i32,
            total_adjustments: adj_row.get::<i64, _>("total") as i32,
            pending_adjustments: adj_row.get::<i64, _>("pending") as i32,
            total_variances: var_row.get::<i64, _>("total") as i32,
            unfavorable_variances: var_row.get::<i64, _>("unfavorable") as i32,
            total_standard_cost_value: fmt2(total_sc_value),
            total_variance_amount: fmt2(total_variance),
            variance_by_type,
            variance_by_element,
            adjustments_by_status,
        })
    }
}
