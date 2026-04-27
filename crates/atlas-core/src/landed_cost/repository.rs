//! Landed Cost Repository
//!
//! PostgreSQL storage for landed cost templates, components, charges,
//! charge lines, allocations, and simulations.

use atlas_shared::{
    LandedCostTemplate, LandedCostComponent, LandedCostCharge,
    LandedCostChargeLine, LandedCostAllocation, LandedCostSimulation,
    AtlasError, AtlasResult,
};
use crate::landed_cost::engine::ReceiptLineInfo;
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for landed cost data storage
#[async_trait]
pub trait LandedCostRepository: Send + Sync {
    // Templates
    async fn create_template(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostTemplate>;
    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LandedCostTemplate>>;
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<LandedCostTemplate>>;
    async fn update_template_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostTemplate>;

    // Components
    async fn create_component(
        &self, org_id: Uuid, template_id: Option<Uuid>, code: &str, name: &str,
        description: Option<&str>, cost_type: &str, allocation_basis: &str,
        default_rate: Option<&str>, rate_uom: Option<&str>, expense_account: Option<&str>,
        is_taxable: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostComponent>;
    async fn get_component(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LandedCostComponent>>;
    async fn list_components(&self, org_id: Uuid, template_id: Option<Uuid>) -> AtlasResult<Vec<LandedCostComponent>>;
    async fn update_component_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostComponent>;

    // Charges
    async fn create_charge(
        &self, org_id: Uuid, charge_number: &str, template_id: Option<Uuid>,
        receipt_id: Option<Uuid>, purchase_order_id: Option<Uuid>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        charge_type: &str, charge_date: Option<chrono::NaiveDate>,
        total_amount: &str, currency: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostCharge>;
    async fn get_charge(&self, id: Uuid) -> AtlasResult<Option<LandedCostCharge>>;
    async fn get_charge_by_number(&self, org_id: Uuid, charge_number: &str) -> AtlasResult<Option<LandedCostCharge>>;
    async fn list_charges(
        &self, org_id: Uuid, status: Option<&str>, charge_type: Option<&str>,
        receipt_id: Option<Uuid>,
    ) -> AtlasResult<Vec<LandedCostCharge>>;
    async fn update_charge_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostCharge>;

    // Charge Lines
    async fn create_charge_line(
        &self, org_id: Uuid, charge_id: Uuid, component_id: Option<Uuid>,
        line_number: i32, receipt_line_id: Option<Uuid>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        charge_amount: &str, allocation_basis: &str,
        allocation_qty: Option<&str>, allocation_value: Option<&str>,
        expense_account: Option<&str>, notes: Option<&str>,
    ) -> AtlasResult<LandedCostChargeLine>;
    async fn get_charge_line(&self, id: Uuid) -> AtlasResult<Option<LandedCostChargeLine>>;
    async fn list_charge_lines(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostChargeLine>>;

    // Allocations
    async fn create_allocation(
        &self, org_id: Uuid, charge_id: Uuid, charge_line_id: Uuid,
        receipt_id: Option<Uuid>, receipt_line_id: Option<Uuid>,
        item_id: Option<Uuid>, item_code: Option<&str>,
        allocated_amount: &str, allocation_basis: &str,
        allocation_basis_value: Option<&str>, total_basis_value: Option<&str>,
        allocation_pct: Option<&str>, original_unit_cost: Option<&str>,
    ) -> AtlasResult<LandedCostAllocation>;
    async fn list_allocations(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>>;
    async fn list_allocations_for_org(&self, org_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>>;
    async fn get_allocations_for_receipt(&self, org_id: Uuid, receipt_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>>;
    async fn get_receipt_lines_for_charge(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptLineInfo>>;

    // Simulations
    async fn create_simulation(
        &self, org_id: Uuid, simulation_number: &str, template_id: Option<Uuid>,
        purchase_order_id: Option<Uuid>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        estimated_quantity: &str, unit_price: &str, currency: &str,
        estimated_charges: &serde_json::Value,
        estimated_landed_cost: &str, estimated_landed_cost_per_unit: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostSimulation>;
    async fn get_simulation(&self, id: Uuid) -> AtlasResult<Option<LandedCostSimulation>>;
    async fn list_simulations(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LandedCostSimulation>>;
    async fn update_simulation_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostSimulation>;
}

/// PostgreSQL implementation
pub struct PostgresLandedCostRepository {
    pool: PgPool,
}

impl PostgresLandedCostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    if v == v.floor() {
        format!("{:.0}", v)
    } else {
        let s = format!("{:.10}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> LandedCostTemplate {
    LandedCostTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_component(row: &sqlx::postgres::PgRow) -> LandedCostComponent {
    LandedCostComponent {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        cost_type: row.get("cost_type"),
        allocation_basis: row.get("allocation_basis"),
        default_rate: row.try_get::<Option<f64>, _>("default_rate").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        rate_uom: row.get("rate_uom"),
        expense_account: row.get("expense_account"),
        is_taxable: row.get("is_taxable"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_charge(row: &sqlx::postgres::PgRow) -> LandedCostCharge {
    LandedCostCharge {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        charge_number: row.get("charge_number"),
        template_id: row.get("template_id"),
        receipt_id: row.get("receipt_id"),
        purchase_order_id: row.get("purchase_order_id"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        charge_type: row.get("charge_type"),
        charge_date: row.get("charge_date"),
        total_amount: get_num(row, "total_amount"),
        currency: row.get("currency"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_charge_line(row: &sqlx::postgres::PgRow) -> LandedCostChargeLine {
    LandedCostChargeLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        charge_id: row.get("charge_id"),
        component_id: row.get("component_id"),
        line_number: row.get("line_number"),
        receipt_line_id: row.get("receipt_line_id"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        charge_amount: get_num(row, "charge_amount"),
        allocated_amount: get_num(row, "allocated_amount"),
        allocation_basis: row.get("allocation_basis"),
        allocation_qty: row.try_get::<Option<f64>, _>("allocation_qty").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        allocation_value: row.try_get::<Option<f64>, _>("allocation_value").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        expense_account: row.get("expense_account"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_allocation(row: &sqlx::postgres::PgRow) -> LandedCostAllocation {
    LandedCostAllocation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        charge_id: row.get("charge_id"),
        charge_line_id: row.get("charge_line_id"),
        receipt_id: row.get("receipt_id"),
        receipt_line_id: row.get("receipt_line_id"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        allocated_amount: get_num(row, "allocated_amount"),
        allocation_basis: row.get("allocation_basis"),
        allocation_basis_value: row.try_get::<Option<f64>, _>("allocation_basis_value").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        total_basis_value: row.try_get::<Option<f64>, _>("total_basis_value").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        allocation_pct: row.try_get::<Option<f64>, _>("allocation_pct").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.4}", v) }),
        unit_landed_cost: row.try_get::<Option<f64>, _>("unit_landed_cost").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        original_unit_cost: row.try_get::<Option<f64>, _>("original_unit_cost").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_simulation(row: &sqlx::postgres::PgRow) -> LandedCostSimulation {
    LandedCostSimulation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        simulation_number: row.get("simulation_number"),
        template_id: row.get("template_id"),
        purchase_order_id: row.get("purchase_order_id"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        estimated_quantity: get_num(row, "estimated_quantity"),
        unit_price: get_num(row, "unit_price"),
        currency: row.get("currency"),
        estimated_charges: row.try_get("estimated_charges").unwrap_or(serde_json::json!([])),
        estimated_landed_cost: get_num(row, "estimated_landed_cost"),
        estimated_landed_cost_per_unit: get_num(row, "estimated_landed_cost_per_unit"),
        variance_vs_actual: row.try_get::<Option<f64>, _>("variance_vs_actual").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.6}", v) }),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl LandedCostRepository for PostgresLandedCostRepository {
    // ========================================================================
    // Templates
    // ========================================================================

    async fn create_template(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostTemplate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.landed_cost_templates
                (organization_id, code, name, description, created_by)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LandedCostTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_templates WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<LandedCostTemplate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_templates WHERE organization_id = $1 ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template).collect())
    }

    async fn update_template_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostTemplate> {
        let row = sqlx::query(
            "UPDATE _atlas.landed_cost_templates SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    // ========================================================================
    // Components
    // ========================================================================

    async fn create_component(
        &self, org_id: Uuid, template_id: Option<Uuid>, code: &str, name: &str,
        description: Option<&str>, cost_type: &str, allocation_basis: &str,
        default_rate: Option<&str>, rate_uom: Option<&str>, expense_account: Option<&str>,
        is_taxable: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostComponent> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.landed_cost_components
                (organization_id, template_id, code, name, description,
                 cost_type, allocation_basis, default_rate, rate_uom,
                 expense_account, is_taxable, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $4, description = $5, cost_type = $6,
                    allocation_basis = $7, default_rate = $8, rate_uom = $9,
                    expense_account = $10, is_taxable = $11, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(template_id).bind(code).bind(name).bind(description)
        .bind(cost_type).bind(allocation_basis)
        .bind(default_rate.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(rate_uom).bind(expense_account).bind(is_taxable).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_component(&row))
    }

    async fn get_component(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LandedCostComponent>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_components WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_component(&r)))
    }

    async fn list_components(&self, org_id: Uuid, template_id: Option<Uuid>) -> AtlasResult<Vec<LandedCostComponent>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.landed_cost_components
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR template_id = $2)
            ORDER BY code
            "#,
        )
        .bind(org_id).bind(template_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_component).collect())
    }

    async fn update_component_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostComponent> {
        let row = sqlx::query(
            "UPDATE _atlas.landed_cost_components SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_component(&row))
    }

    // ========================================================================
    // Charges
    // ========================================================================

    async fn create_charge(
        &self, org_id: Uuid, charge_number: &str, template_id: Option<Uuid>,
        receipt_id: Option<Uuid>, purchase_order_id: Option<Uuid>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        charge_type: &str, charge_date: Option<chrono::NaiveDate>,
        total_amount: &str, currency: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostCharge> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.landed_cost_charges
                (organization_id, charge_number, template_id, receipt_id,
                 purchase_order_id, supplier_id, supplier_name,
                 charge_type, charge_date, total_amount, currency, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(charge_number).bind(template_id).bind(receipt_id)
        .bind(purchase_order_id).bind(supplier_id).bind(supplier_name)
        .bind(charge_type).bind(charge_date)
        .bind(total_amount.parse::<f64>().unwrap_or(0.0))
        .bind(currency).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("violates unique constraint") {
                AtlasError::Conflict(format!("Charge number '{}' already exists", charge_number))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;
        Ok(row_to_charge(&row))
    }

    async fn get_charge(&self, id: Uuid) -> AtlasResult<Option<LandedCostCharge>> {
        let row = sqlx::query("SELECT * FROM _atlas.landed_cost_charges WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_charge(&r)))
    }

    async fn get_charge_by_number(&self, org_id: Uuid, charge_number: &str) -> AtlasResult<Option<LandedCostCharge>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_charges WHERE organization_id = $1 AND charge_number = $2"
        )
        .bind(org_id).bind(charge_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_charge(&r)))
    }

    async fn list_charges(
        &self, org_id: Uuid, status: Option<&str>, charge_type: Option<&str>,
        receipt_id: Option<Uuid>,
    ) -> AtlasResult<Vec<LandedCostCharge>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.landed_cost_charges
            WHERE organization_id = $1
              AND ($2 IS NULL OR status = $2)
              AND ($3 IS NULL OR charge_type = $3)
              AND ($4::uuid IS NULL OR receipt_id = $4)
            ORDER BY charge_date DESC NULLS LAST, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(charge_type).bind(receipt_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_charge).collect())
    }

    async fn update_charge_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostCharge> {
        let row = sqlx::query(
            "UPDATE _atlas.landed_cost_charges SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_charge(&row))
    }

    // ========================================================================
    // Charge Lines
    // ========================================================================

    async fn create_charge_line(
        &self, org_id: Uuid, charge_id: Uuid, component_id: Option<Uuid>,
        line_number: i32, receipt_line_id: Option<Uuid>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        charge_amount: &str, allocation_basis: &str,
        allocation_qty: Option<&str>, allocation_value: Option<&str>,
        expense_account: Option<&str>, notes: Option<&str>,
    ) -> AtlasResult<LandedCostChargeLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.landed_cost_charge_lines
                (organization_id, charge_id, component_id, line_number,
                 receipt_line_id, item_id, item_code, item_description,
                 charge_amount, allocation_basis, allocation_qty, allocation_value,
                 expense_account, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(charge_id).bind(component_id).bind(line_number)
        .bind(receipt_line_id).bind(item_id).bind(item_code).bind(item_description)
        .bind(charge_amount.parse::<f64>().unwrap_or(0.0)).bind(allocation_basis)
        .bind(allocation_qty.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(allocation_value.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(expense_account).bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_charge_line(&row))
    }

    async fn get_charge_line(&self, id: Uuid) -> AtlasResult<Option<LandedCostChargeLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.landed_cost_charge_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_charge_line(&r)))
    }

    async fn list_charge_lines(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostChargeLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_charge_lines WHERE charge_id = $1 ORDER BY line_number"
        )
        .bind(charge_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_charge_line).collect())
    }

    // ========================================================================
    // Allocations
    // ========================================================================

    async fn create_allocation(
        &self, org_id: Uuid, charge_id: Uuid, charge_line_id: Uuid,
        receipt_id: Option<Uuid>, receipt_line_id: Option<Uuid>,
        item_id: Option<Uuid>, item_code: Option<&str>,
        allocated_amount: &str, allocation_basis: &str,
        allocation_basis_value: Option<&str>, total_basis_value: Option<&str>,
        allocation_pct: Option<&str>, original_unit_cost: Option<&str>,
    ) -> AtlasResult<LandedCostAllocation> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.landed_cost_allocations
                (organization_id, charge_id, charge_line_id,
                 receipt_id, receipt_line_id, item_id, item_code,
                 allocated_amount, allocation_basis, allocation_basis_value,
                 total_basis_value, allocation_pct, original_unit_cost)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(charge_id).bind(charge_line_id)
        .bind(receipt_id).bind(receipt_line_id).bind(item_id).bind(item_code)
        .bind(allocated_amount.parse::<f64>().unwrap_or(0.0)).bind(allocation_basis)
        .bind(allocation_basis_value.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(total_basis_value.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(allocation_pct.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(original_unit_cost.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_allocation(&row))
    }

    async fn list_allocations(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_allocations WHERE charge_id = $1 ORDER BY created_at"
        )
        .bind(charge_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_allocation).collect())
    }

    async fn list_allocations_for_org(&self, org_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_allocations WHERE organization_id = $1 ORDER BY created_at"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_allocation).collect())
    }

    async fn get_allocations_for_receipt(&self, org_id: Uuid, receipt_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.landed_cost_allocations WHERE organization_id = $1 AND receipt_id = $2 ORDER BY created_at"
        )
        .bind(org_id).bind(receipt_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_allocation).collect())
    }

    async fn get_receipt_lines_for_charge(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptLineInfo>> {
        let Some(rid) = receipt_id else {
            return Ok(vec![]);
        };

        let rows = sqlx::query(
            r#"
            SELECT id as receipt_line_id, item_id, item_code,
                   received_qty as quantity, unit_price,
                   0.0 as weight, 0.0 as volume
            FROM _atlas.receipt_lines
            WHERE receipt_id = $1
              AND organization_id = $2
            ORDER BY line_number
            "#,
        )
        .bind(rid).bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| ReceiptLineInfo {
            receipt_line_id: r.get("receipt_line_id"),
            item_id: r.get("item_id"),
            item_code: r.get("item_code"),
            quantity: r.try_get::<f64, _>("quantity").unwrap_or(0.0),
            unit_price: r.try_get::<Option<f64>, _>("unit_price").ok().flatten().map(|v| format!("{:.2}", v)),
            weight: r.try_get::<Option<f64>, _>("weight").ok().flatten(),
            volume: r.try_get::<Option<f64>, _>("volume").ok().flatten(),
        }).collect())
    }

    // ========================================================================
    // Simulations
    // ========================================================================

    async fn create_simulation(
        &self, org_id: Uuid, simulation_number: &str, template_id: Option<Uuid>,
        purchase_order_id: Option<Uuid>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        estimated_quantity: &str, unit_price: &str, currency: &str,
        estimated_charges: &serde_json::Value,
        estimated_landed_cost: &str, estimated_landed_cost_per_unit: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostSimulation> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.landed_cost_simulations
                (organization_id, simulation_number, template_id,
                 purchase_order_id, item_id, item_code, item_description,
                 estimated_quantity, unit_price, currency,
                 estimated_charges, estimated_landed_cost, estimated_landed_cost_per_unit,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(simulation_number).bind(template_id)
        .bind(purchase_order_id).bind(item_id).bind(item_code).bind(item_description)
        .bind(estimated_quantity.parse::<f64>().unwrap_or(0.0))
        .bind(unit_price.parse::<f64>().unwrap_or(0.0))
        .bind(currency).bind(estimated_charges)
        .bind(estimated_landed_cost.parse::<f64>().unwrap_or(0.0))
        .bind(estimated_landed_cost_per_unit.parse::<f64>().unwrap_or(0.0))
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("violates unique constraint") {
                AtlasError::Conflict(format!("Simulation number '{}' already exists", simulation_number))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;
        Ok(row_to_simulation(&row))
    }

    async fn get_simulation(&self, id: Uuid) -> AtlasResult<Option<LandedCostSimulation>> {
        let row = sqlx::query("SELECT * FROM _atlas.landed_cost_simulations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_simulation(&r)))
    }

    async fn list_simulations(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LandedCostSimulation>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.landed_cost_simulations
            WHERE organization_id = $1 AND ($2 IS NULL OR status = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_simulation).collect())
    }

    async fn update_simulation_status(&self, id: Uuid, status: &str) -> AtlasResult<LandedCostSimulation> {
        let row = sqlx::query(
            "UPDATE _atlas.landed_cost_simulations SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_simulation(&row))
    }
}
