//! Shipping Execution Repository
//!
//! PostgreSQL storage for shipping execution data.

use atlas_shared::{
    ShippingCarrier, ShippingMethod, Shipment, ShipmentLine,
    PackingSlip, PackingSlipLine, ShippingDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for Shipping Execution data storage
#[async_trait]
pub trait ShippingRepository: Send + Sync {
    // Carriers
    async fn create_carrier(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        carrier_type: &str, tracking_url_template: Option<&str>,
        contact_name: Option<&str>, contact_phone: Option<&str>, contact_email: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ShippingCarrier>;
    async fn get_carrier(&self, id: Uuid) -> AtlasResult<Option<ShippingCarrier>>;
    async fn get_carrier_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ShippingCarrier>>;
    async fn list_carriers(&self, org_id: Uuid) -> AtlasResult<Vec<ShippingCarrier>>;
    async fn delete_carrier(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Shipping Methods
    async fn create_method(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        carrier_id: Option<Uuid>, transit_time_days: i32, is_express: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ShippingMethod>;
    async fn get_method(&self, id: Uuid) -> AtlasResult<Option<ShippingMethod>>;
    async fn list_methods(&self, org_id: Uuid) -> AtlasResult<Vec<ShippingMethod>>;
    async fn delete_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Shipments
    async fn create_shipment(
        &self, org_id: Uuid, shipment_number: &str, description: Option<&str>,
        carrier_id: Option<Uuid>, carrier_name: Option<&str>,
        shipping_method_id: Option<Uuid>, shipping_method_name: Option<&str>,
        order_id: Option<Uuid>, order_number: Option<&str>,
        customer_id: Option<Uuid>, customer_name: Option<&str>,
        ship_from_warehouse: Option<&str>,
        ship_to_name: Option<&str>, ship_to_address: Option<&str>,
        ship_to_city: Option<&str>, ship_to_state: Option<&str>,
        ship_to_postal_code: Option<&str>, ship_to_country: Option<&str>,
        estimated_delivery: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Shipment>;
    async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<Shipment>>;
    async fn get_shipment_by_number(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<Option<Shipment>>;
    async fn list_shipments(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<Shipment>>;
    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<Shipment>;
    async fn confirm_shipment(&self, id: Uuid, confirmed_by: Option<Uuid>) -> AtlasResult<Shipment>;
    async fn ship_confirm(
        &self, id: Uuid, tracking_number: Option<&str>,
        shipped_by: Option<Uuid>,
    ) -> AtlasResult<Shipment>;
    async fn deliver(&self, id: Uuid, delivered_by: Option<Uuid>) -> AtlasResult<Shipment>;
    async fn delete_shipment(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<()>;

    // Shipment Lines
    async fn add_shipment_line(
        &self, org_id: Uuid, shipment_id: Uuid, line_number: i32,
        order_line_id: Option<Uuid>, item_code: &str, item_name: Option<&str>,
        item_description: Option<&str>, requested_quantity: &str,
        unit_of_measure: Option<&str>, weight: Option<&str>,
        weight_unit: Option<&str>, lot_number: Option<&str>,
        serial_number: Option<&str>, is_fragile: bool, is_hazardous: bool,
        notes: Option<&str>,
    ) -> AtlasResult<ShipmentLine>;
    async fn list_shipment_lines(&self, shipment_id: Uuid) -> AtlasResult<Vec<ShipmentLine>>;
    async fn get_shipment_line(&self, id: Uuid) -> AtlasResult<Option<ShipmentLine>>;
    async fn delete_shipment_line(&self, id: Uuid) -> AtlasResult<()>;
    async fn update_line_shipped_quantity(&self, id: Uuid, shipped_qty: &str) -> AtlasResult<ShipmentLine>;

    // Packing Slips
    async fn create_packing_slip(
        &self, org_id: Uuid, shipment_id: Uuid, packing_slip_number: &str,
        package_number: i32, package_type: Option<&str>,
        weight: Option<&str>, weight_unit: Option<&str>,
        dimensions_length: Option<&str>, dimensions_width: Option<&str>,
        dimensions_height: Option<&str>, dimensions_unit: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<PackingSlip>;
    async fn list_packing_slips(&self, shipment_id: Uuid) -> AtlasResult<Vec<PackingSlip>>;
    async fn get_packing_slip(&self, id: Uuid) -> AtlasResult<Option<PackingSlip>>;
    async fn delete_packing_slip(&self, id: Uuid) -> AtlasResult<()>;

    // Packing Slip Lines
    async fn add_packing_slip_line(
        &self, org_id: Uuid, packing_slip_id: Uuid, shipment_line_id: Uuid,
        line_number: i32, item_code: &str, item_name: Option<&str>,
        packed_quantity: &str, notes: Option<&str>,
    ) -> AtlasResult<PackingSlipLine>;
    async fn list_packing_slip_lines(&self, packing_slip_id: Uuid) -> AtlasResult<Vec<PackingSlipLine>>;
    async fn delete_packing_slip_line(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ShippingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresShippingRepository {
    pool: PgPool,
}

impl PostgresShippingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_carrier(row: &sqlx::postgres::PgRow) -> ShippingCarrier {
    ShippingCarrier {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        carrier_type: row.get("carrier_type"),
        tracking_url_template: row.get("tracking_url_template"),
        contact_name: row.get("contact_name"),
        contact_phone: row.get("contact_phone"),
        contact_email: row.get("contact_email"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_method(row: &sqlx::postgres::PgRow) -> ShippingMethod {
    ShippingMethod {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        carrier_id: row.get("carrier_id"),
        transit_time_days: row.get("transit_time_days"),
        is_express: row.get("is_express"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_shipment(row: &sqlx::postgres::PgRow) -> Shipment {
    Shipment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        shipment_number: row.get("shipment_number"),
        description: row.get("description"),
        status: row.get("status"),
        carrier_id: row.get("carrier_id"),
        carrier_name: row.get("carrier_name"),
        shipping_method_id: row.get("shipping_method_id"),
        shipping_method_name: row.get("shipping_method_name"),
        order_id: row.get("order_id"),
        order_number: row.get("order_number"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        ship_from_warehouse: row.get("ship_from_warehouse"),
        ship_to_name: row.get("ship_to_name"),
        ship_to_address: row.get("ship_to_address"),
        ship_to_city: row.get("ship_to_city"),
        ship_to_state: row.get("ship_to_state"),
        ship_to_postal_code: row.get("ship_to_postal_code"),
        ship_to_country: row.get("ship_to_country"),
        tracking_number: row.get("tracking_number"),
        total_weight: get_num(row, "total_weight"),
        weight_unit: row.try_get("weight_unit").unwrap_or_else(|_| "kg".to_string()),
        total_volume: get_num(row, "total_volume"),
        volume_unit: row.try_get("volume_unit").unwrap_or_else(|_| "m3".to_string()),
        total_packages: row.get("total_packages"),
        shipped_date: row.get("shipped_date"),
        estimated_delivery: row.get("estimated_delivery"),
        actual_delivery: row.get("actual_delivery"),
        confirmed_by: row.get("confirmed_by"),
        confirmed_at: row.get("confirmed_at"),
        shipped_by: row.get("shipped_by"),
        delivered_by: row.get("delivered_by"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_shipment_line(row: &sqlx::postgres::PgRow) -> ShipmentLine {
    ShipmentLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        shipment_id: row.get("shipment_id"),
        line_number: row.get("line_number"),
        order_line_id: row.get("order_line_id"),
        item_code: row.get("item_code"),
        item_name: row.get("item_name"),
        item_description: row.get("item_description"),
        requested_quantity: get_num(row, "requested_quantity"),
        shipped_quantity: get_num(row, "shipped_quantity"),
        backordered_quantity: get_num(row, "backordered_quantity"),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_else(|_| "EA".to_string()),
        weight: get_num(row, "weight"),
        weight_unit: row.try_get("weight_unit").unwrap_or_else(|_| "kg".to_string()),
        lot_number: row.get("lot_number"),
        serial_number: row.get("serial_number"),
        is_fragile: row.get("is_fragile"),
        is_hazardous: row.get("is_hazardous"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_packing_slip(row: &sqlx::postgres::PgRow) -> PackingSlip {
    PackingSlip {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        shipment_id: row.get("shipment_id"),
        packing_slip_number: row.get("packing_slip_number"),
        package_number: row.get("package_number"),
        package_type: row.try_get("package_type").unwrap_or_else(|_| "box".to_string()),
        weight: get_num(row, "weight"),
        weight_unit: row.try_get("weight_unit").unwrap_or_else(|_| "kg".to_string()),
        dimensions_length: get_num(row, "dimensions_length"),
        dimensions_width: get_num(row, "dimensions_width"),
        dimensions_height: get_num(row, "dimensions_height"),
        dimensions_unit: row.try_get("dimensions_unit").unwrap_or_else(|_| "cm".to_string()),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_packing_slip_line(row: &sqlx::postgres::PgRow) -> PackingSlipLine {
    PackingSlipLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        packing_slip_id: row.get("packing_slip_id"),
        shipment_line_id: row.get("shipment_line_id"),
        line_number: row.get("line_number"),
        item_code: row.get("item_code"),
        item_name: row.get("item_name"),
        packed_quantity: get_num(row, "packed_quantity"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl ShippingRepository for PostgresShippingRepository {
    // ========================================================================
    // Carriers
    // ========================================================================

    async fn create_carrier(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        carrier_type: &str, tracking_url_template: Option<&str>,
        contact_name: Option<&str>, contact_phone: Option<&str>, contact_email: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ShippingCarrier> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipping_carriers
                (organization_id, code, name, description, carrier_type,
                 tracking_url_template, contact_name, contact_phone, contact_email, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(carrier_type).bind(tracking_url_template)
        .bind(contact_name).bind(contact_phone).bind(contact_email)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_carrier(&row))
    }

    async fn get_carrier(&self, id: Uuid) -> AtlasResult<Option<ShippingCarrier>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.shipping_carriers WHERE id = $1 AND is_active = true",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_carrier(&r)))
    }

    async fn get_carrier_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ShippingCarrier>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.shipping_carriers WHERE organization_id = $1 AND code = $2 AND is_active = true",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_carrier(&r)))
    }

    async fn list_carriers(&self, org_id: Uuid) -> AtlasResult<Vec<ShippingCarrier>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.shipping_carriers WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_carrier).collect())
    }

    async fn delete_carrier(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.shipping_carriers WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Shipping Methods
    // ========================================================================

    async fn create_method(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        carrier_id: Option<Uuid>, transit_time_days: i32, is_express: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ShippingMethod> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipping_methods
                (organization_id, code, name, description, carrier_id, transit_time_days, is_express, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(carrier_id).bind(transit_time_days).bind(is_express).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_method(&row))
    }

    async fn get_method(&self, id: Uuid) -> AtlasResult<Option<ShippingMethod>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.shipping_methods WHERE id = $1 AND is_active = true",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_method(&r)))
    }

    async fn list_methods(&self, org_id: Uuid) -> AtlasResult<Vec<ShippingMethod>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.shipping_methods WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_method).collect())
    }

    async fn delete_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.shipping_methods WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Shipments
    // ========================================================================

    async fn create_shipment(
        &self, org_id: Uuid, shipment_number: &str, description: Option<&str>,
        carrier_id: Option<Uuid>, carrier_name: Option<&str>,
        shipping_method_id: Option<Uuid>, shipping_method_name: Option<&str>,
        order_id: Option<Uuid>, order_number: Option<&str>,
        customer_id: Option<Uuid>, customer_name: Option<&str>,
        ship_from_warehouse: Option<&str>,
        ship_to_name: Option<&str>, ship_to_address: Option<&str>,
        ship_to_city: Option<&str>, ship_to_state: Option<&str>,
        ship_to_postal_code: Option<&str>, ship_to_country: Option<&str>,
        estimated_delivery: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Shipment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipments
                (organization_id, shipment_number, description,
                 carrier_id, carrier_name, shipping_method_id, shipping_method_name,
                 order_id, order_number, customer_id, customer_name,
                 ship_from_warehouse,
                 ship_to_name, ship_to_address, ship_to_city, ship_to_state,
                 ship_to_postal_code, ship_to_country,
                 estimated_delivery, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21) RETURNING *"#,
        )
        .bind(org_id).bind(shipment_number).bind(description)
        .bind(carrier_id).bind(carrier_name)
        .bind(shipping_method_id).bind(shipping_method_name)
        .bind(order_id).bind(order_number)
        .bind(customer_id).bind(customer_name)
        .bind(ship_from_warehouse)
        .bind(ship_to_name).bind(ship_to_address)
        .bind(ship_to_city).bind(ship_to_state)
        .bind(ship_to_postal_code).bind(ship_to_country)
        .bind(estimated_delivery).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment(&row))
    }

    async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<Shipment>> {
        let row = sqlx::query("SELECT * FROM _atlas.shipments WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_shipment(&r)))
    }

    async fn get_shipment_by_number(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<Option<Shipment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.shipments WHERE organization_id = $1 AND shipment_number = $2",
        )
        .bind(org_id).bind(shipment_number).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_shipment(&r)))
    }

    async fn list_shipments(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<Shipment>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.shipments WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.shipments WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_shipment).collect())
    }

    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<Shipment> {
        let row = sqlx::query(
            "UPDATE _atlas.shipments SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment(&row))
    }

    async fn confirm_shipment(&self, id: Uuid, confirmed_by: Option<Uuid>) -> AtlasResult<Shipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET status = 'confirmed', confirmed_by = $2, confirmed_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(confirmed_by).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment(&row))
    }

    async fn ship_confirm(
        &self, id: Uuid, tracking_number: Option<&str>,
        shipped_by: Option<Uuid>,
    ) -> AtlasResult<Shipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET status = 'shipped', tracking_number = COALESCE($2, tracking_number),
                   shipped_by = $3, shipped_date = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(tracking_number).bind(shipped_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment(&row))
    }

    async fn deliver(&self, id: Uuid, delivered_by: Option<Uuid>) -> AtlasResult<Shipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET status = 'delivered', delivered_by = $2, actual_delivery = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(delivered_by).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment(&row))
    }

    async fn delete_shipment(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.shipments WHERE organization_id = $1 AND shipment_number = $2")
            .bind(org_id).bind(shipment_number).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Shipment Lines
    // ========================================================================

    async fn add_shipment_line(
        &self, org_id: Uuid, shipment_id: Uuid, line_number: i32,
        order_line_id: Option<Uuid>, item_code: &str, item_name: Option<&str>,
        item_description: Option<&str>, requested_quantity: &str,
        unit_of_measure: Option<&str>, weight: Option<&str>,
        weight_unit: Option<&str>, lot_number: Option<&str>,
        serial_number: Option<&str>, is_fragile: bool, is_hazardous: bool,
        notes: Option<&str>,
    ) -> AtlasResult<ShipmentLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipment_lines
                (organization_id, shipment_id, line_number, order_line_id,
                 item_code, item_name, item_description, requested_quantity,
                 unit_of_measure, weight, weight_unit, lot_number, serial_number,
                 is_fragile, is_hazardous, notes)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) RETURNING *"#,
        )
        .bind(org_id).bind(shipment_id).bind(line_number).bind(order_line_id)
        .bind(item_code).bind(item_name).bind(item_description)
        .bind(requested_quantity.parse::<f64>().unwrap_or(0.0))
        .bind(unit_of_measure).bind(weight.map(|w| w.parse::<f64>().unwrap_or(0.0)))
        .bind(weight_unit).bind(lot_number).bind(serial_number)
        .bind(is_fragile).bind(is_hazardous).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment_line(&row))
    }

    async fn list_shipment_lines(&self, shipment_id: Uuid) -> AtlasResult<Vec<ShipmentLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.shipment_lines WHERE shipment_id = $1 ORDER BY line_number",
        )
        .bind(shipment_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_shipment_line).collect())
    }

    async fn get_shipment_line(&self, id: Uuid) -> AtlasResult<Option<ShipmentLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.shipment_lines WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_shipment_line(&r)))
    }

    async fn delete_shipment_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.shipment_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_line_shipped_quantity(&self, id: Uuid, shipped_qty: &str) -> AtlasResult<ShipmentLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipment_lines
               SET shipped_quantity = $2,
                   backordered_quantity = requested_quantity - $2,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(shipped_qty.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_shipment_line(&row))
    }

    // ========================================================================
    // Packing Slips
    // ========================================================================

    async fn create_packing_slip(
        &self, org_id: Uuid, shipment_id: Uuid, packing_slip_number: &str,
        package_number: i32, package_type: Option<&str>,
        weight: Option<&str>, weight_unit: Option<&str>,
        dimensions_length: Option<&str>, dimensions_width: Option<&str>,
        dimensions_height: Option<&str>, dimensions_unit: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<PackingSlip> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.packing_slips
                (organization_id, shipment_id, packing_slip_number, package_number,
                 package_type, weight, weight_unit,
                 dimensions_length, dimensions_width, dimensions_height, dimensions_unit,
                 notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING *"#,
        )
        .bind(org_id).bind(shipment_id).bind(packing_slip_number).bind(package_number)
        .bind(package_type)
        .bind(weight.map(|w| w.parse::<f64>().unwrap_or(0.0)))
        .bind(weight_unit)
        .bind(dimensions_length.map(|d| d.parse::<f64>().unwrap_or(0.0)))
        .bind(dimensions_width.map(|d| d.parse::<f64>().unwrap_or(0.0)))
        .bind(dimensions_height.map(|d| d.parse::<f64>().unwrap_or(0.0)))
        .bind(dimensions_unit)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_packing_slip(&row))
    }

    async fn list_packing_slips(&self, shipment_id: Uuid) -> AtlasResult<Vec<PackingSlip>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.packing_slips WHERE shipment_id = $1 ORDER BY package_number",
        )
        .bind(shipment_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_packing_slip).collect())
    }

    async fn get_packing_slip(&self, id: Uuid) -> AtlasResult<Option<PackingSlip>> {
        let row = sqlx::query("SELECT * FROM _atlas.packing_slips WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_packing_slip(&r)))
    }

    async fn delete_packing_slip(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.packing_slips WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Packing Slip Lines
    // ========================================================================

    async fn add_packing_slip_line(
        &self, org_id: Uuid, packing_slip_id: Uuid, shipment_line_id: Uuid,
        line_number: i32, item_code: &str, item_name: Option<&str>,
        packed_quantity: &str, notes: Option<&str>,
    ) -> AtlasResult<PackingSlipLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.packing_slip_lines
                (organization_id, packing_slip_id, shipment_line_id,
                 line_number, item_code, item_name, packed_quantity, notes)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *"#,
        )
        .bind(org_id).bind(packing_slip_id).bind(shipment_line_id)
        .bind(line_number).bind(item_code).bind(item_name)
        .bind(packed_quantity.parse::<f64>().unwrap_or(0.0)).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_packing_slip_line(&row))
    }

    async fn list_packing_slip_lines(&self, packing_slip_id: Uuid) -> AtlasResult<Vec<PackingSlipLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.packing_slip_lines WHERE packing_slip_id = $1 ORDER BY line_number",
        )
        .bind(packing_slip_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_packing_slip_line).collect())
    }

    async fn delete_packing_slip_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.packing_slip_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ShippingDashboard> {

        let summary_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('draft', 'confirmed', 'picked', 'packed')) as pending,
                COUNT(*) FILTER (WHERE status = 'shipped' AND shipped_date >= date_trunc('month', now())) as shipped_month,
                COUNT(*) FILTER (WHERE status = 'delivered' AND actual_delivery >= date_trunc('month', now())) as delivered_month
               FROM _atlas.shipments WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let carrier_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.shipping_carriers WHERE organization_id = $1 AND is_active = true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        // By status
        let status_rows = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.shipments
               WHERE organization_id = $1 GROUP BY status ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let shipments_by_status: serde_json::Value = status_rows.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        // Recent shipments
        let recent_rows = sqlx::query(
            "SELECT id, shipment_number, status, carrier_name, customer_name, created_at \
             FROM _atlas.shipments WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 10",
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let recent_shipments: serde_json::Value = recent_rows.iter().map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "shipmentNumber": r.get::<String, _>("shipment_number"),
                "status": r.get::<String, _>("status"),
                "carrierName": r.get::<Option<String>, _>("carrier_name").unwrap_or_default(),
                "customerName": r.get::<Option<String>, _>("customer_name").unwrap_or_default(),
                "createdAt": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            })
        }).collect();

        // Top carriers
        let carrier_rows = sqlx::query(
            r#"SELECT carrier_name, COUNT(*) as cnt FROM _atlas.shipments
               WHERE organization_id = $1 AND carrier_name IS NOT NULL
               GROUP BY carrier_name ORDER BY cnt DESC LIMIT 10"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let top_carriers: serde_json::Value = carrier_rows.iter().map(|r| {
            serde_json::json!({
                "name": r.get::<String, _>("carrier_name"),
                "shipmentCount": r.get::<i64, _>("cnt"),
            })
        }).collect();

        Ok(ShippingDashboard {
            total_shipments: summary_row.get::<i64, _>("total") as i32,
            pending_shipments: summary_row.get::<i64, _>("pending") as i32,
            shipped_this_month: summary_row.get::<i64, _>("shipped_month") as i32,
            delivered_this_month: summary_row.get::<i64, _>("delivered_month") as i32,
            total_carriers: carrier_count as i32,
            shipments_by_status,
            recent_shipments,
            top_carriers,
        })
    }
}
