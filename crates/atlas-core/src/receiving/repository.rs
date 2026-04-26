//! Receiving Repository
//!
//! PostgreSQL storage for receiving locations, receipts, lines,
//! inspections, inspection details, deliveries, and returns.

use atlas_shared::{
    ReceivingLocation, ReceiptHeader, ReceiptLine, ReceiptInspection,
    InspectionDetail, ReceiptDelivery, ReceiptReturn,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for receiving data storage
#[async_trait]
pub trait ReceivingRepository: Send + Sync {
    // Locations
    async fn create_location(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        location_type: &str, address: Option<&str>, city: Option<&str>,
        state: Option<&str>, country: Option<&str>, postal_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceivingLocation>;
    async fn get_location(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ReceivingLocation>>;
    async fn list_locations(&self, org_id: Uuid) -> AtlasResult<Vec<ReceivingLocation>>;
    async fn delete_location(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Receipts
    async fn create_receipt(
        &self, org_id: Uuid, receipt_number: &str, receipt_type: &str,
        receipt_source: &str, supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        supplier_number: Option<&str>, purchase_order_id: Option<Uuid>,
        purchase_order_number: Option<&str>, receiving_location_id: Option<Uuid>,
        receiving_location_code: Option<&str>, receiving_date: chrono::NaiveDate,
        packing_slip_number: Option<&str>, bill_of_lading: Option<&str>,
        carrier: Option<&str>, tracking_number: Option<&str>,
        waybill_number: Option<&str>, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptHeader>;
    async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<ReceiptHeader>>;
    async fn get_receipt_by_number(&self, org_id: Uuid, receipt_number: &str) -> AtlasResult<Option<ReceiptHeader>>;
    async fn list_receipts(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptHeader>>;
    async fn update_receipt_status(
        &self, id: Uuid, status: &str, received_by: Option<Uuid>, closed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ReceiptHeader>;

    // Receipt Lines
    async fn create_receipt_line(
        &self, org_id: Uuid, receipt_id: Uuid, line_number: i32,
        purchase_order_line_id: Option<Uuid>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        ordered_qty: &str, ordered_uom: Option<&str>,
        received_qty: &str, received_uom: Option<&str>,
        lot_number: Option<&str>, serial_numbers: serde_json::Value,
        expiration_date: Option<chrono::NaiveDate>, manufacture_date: Option<chrono::NaiveDate>,
        unit_price: Option<&str>, currency: Option<&str>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptLine>;
    async fn get_receipt_line(&self, id: Uuid) -> AtlasResult<Option<ReceiptLine>>;
    async fn list_receipt_lines(&self, receipt_id: Uuid) -> AtlasResult<Vec<ReceiptLine>>;

    // Inspections
    async fn create_inspection(
        &self, org_id: Uuid, receipt_id: Uuid, receipt_line_id: Uuid,
        inspection_number: &str, inspection_template: Option<&str>,
        inspector_id: Option<Uuid>, inspector_name: Option<&str>,
        inspection_date: chrono::NaiveDate, sample_size: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptInspection>;
    async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<ReceiptInspection>>;
    async fn list_inspections(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptInspection>>;
    async fn complete_inspection(
        &self, id: Uuid, quantity_inspected: &str, quantity_accepted: &str,
        quantity_rejected: &str, disposition: &str, quality_score: Option<&str>,
        rejection_reason: Option<&str>, notes: Option<&str>,
    ) -> AtlasResult<ReceiptInspection>;

    // Inspection Details
    async fn create_inspection_detail(
        &self, org_id: Uuid, inspection_id: Uuid, check_number: i32,
        check_name: &str, check_type: &str, specification: Option<&str>,
        result: &str, measured_value: Option<&str>, expected_value: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<InspectionDetail>;
    async fn list_inspection_details(&self, inspection_id: Uuid) -> AtlasResult<Vec<InspectionDetail>>;

    // Deliveries
    async fn create_delivery(
        &self, org_id: Uuid, receipt_id: Uuid, receipt_line_id: Uuid,
        delivery_number: &str, subinventory: Option<&str>, locator: Option<&str>,
        quantity_delivered: &str, uom: Option<&str>, lot_number: Option<&str>,
        serial_number: Option<&str>, delivered_by: Option<Uuid>,
        delivered_by_name: Option<&str>, destination_type: &str,
        account_code: Option<&str>, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptDelivery>;
    async fn get_delivery(&self, id: Uuid) -> AtlasResult<Option<ReceiptDelivery>>;
    async fn list_deliveries(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptDelivery>>;

    // Returns
    async fn create_return(
        &self, org_id: Uuid, return_number: &str, receipt_id: Option<Uuid>,
        receipt_line_id: Option<Uuid>, supplier_id: Option<Uuid>,
        supplier_name: Option<&str>, return_type: &str,
        item_id: Option<Uuid>, item_code: Option<&str>,
        item_description: Option<&str>, quantity_returned: &str,
        uom: Option<&str>, unit_price: Option<&str>, currency: Option<&str>,
        return_reason: Option<&str>, return_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptReturn>;
    async fn get_return(&self, id: Uuid) -> AtlasResult<Option<ReceiptReturn>>;
    async fn list_returns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptReturn>>;
    async fn update_return_status(
        &self, id: Uuid, status: &str, carrier: Option<&str>, tracking_number: Option<&str>,
    ) -> AtlasResult<ReceiptReturn>;
}

/// PostgreSQL implementation
pub struct PostgresReceivingRepository {
    pool: PgPool,
}

impl PostgresReceivingRepository {
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

fn row_to_location(row: &sqlx::postgres::PgRow) -> ReceivingLocation {
    ReceivingLocation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        location_type: row.get("location_type"),
        address: row.get("address"),
        city: row.get("city"),
        state: row.get("state"),
        country: row.get("country"),
        postal_code: row.get("postal_code"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_receipt(row: &sqlx::postgres::PgRow) -> ReceiptHeader {
    ReceiptHeader {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        receipt_number: row.get("receipt_number"),
        receipt_type: row.get("receipt_type"),
        receipt_source: row.get("receipt_source"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        supplier_number: row.get("supplier_number"),
        purchase_order_id: row.get("purchase_order_id"),
        purchase_order_number: row.get("purchase_order_number"),
        receiving_location_id: row.get("receiving_location_id"),
        receiving_location_code: row.get("receiving_location_code"),
        receiving_date: row.get("receiving_date"),
        packing_slip_number: row.get("packing_slip_number"),
        bill_of_lading: row.get("bill_of_lading"),
        carrier: row.get("carrier"),
        tracking_number: row.get("tracking_number"),
        waybill_number: row.get("waybill_number"),
        notes: row.get("notes"),
        status: row.get("status"),
        total_received_qty: get_num(row, "total_received_qty"),
        total_inspected_qty: get_num(row, "total_inspected_qty"),
        total_accepted_qty: get_num(row, "total_accepted_qty"),
        total_rejected_qty: get_num(row, "total_rejected_qty"),
        total_delivered_qty: get_num(row, "total_delivered_qty"),
        received_by: row.get("received_by"),
        received_at: row.get("received_at"),
        closed_at: row.get("closed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_line(row: &sqlx::postgres::PgRow) -> ReceiptLine {
    ReceiptLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        receipt_id: row.get("receipt_id"),
        line_number: row.get("line_number"),
        purchase_order_line_id: row.get("purchase_order_line_id"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        ordered_qty: get_num(row, "ordered_qty"),
        ordered_uom: row.get("ordered_uom"),
        received_qty: get_num(row, "received_qty"),
        received_uom: row.get("received_uom"),
        accepted_qty: get_num(row, "accepted_qty"),
        rejected_qty: get_num(row, "rejected_qty"),
        inspection_status: row.get("inspection_status"),
        delivery_status: row.get("delivery_status"),
        lot_number: row.get("lot_number"),
        serial_numbers: row.try_get("serial_numbers").unwrap_or(serde_json::json!([])),
        expiration_date: row.get("expiration_date"),
        manufacture_date: row.get("manufacture_date"),
        unit_price: row.try_get::<Option<f64>, _>("unit_price").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.2}", v) }),
        currency: row.get("currency"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_inspection(row: &sqlx::postgres::PgRow) -> ReceiptInspection {
    ReceiptInspection {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        receipt_id: row.get("receipt_id"),
        receipt_line_id: row.get("receipt_line_id"),
        inspection_number: row.get("inspection_number"),
        inspection_template: row.get("inspection_template"),
        inspector_id: row.get("inspector_id"),
        inspector_name: row.get("inspector_name"),
        inspection_date: row.get("inspection_date"),
        sample_size: row.try_get::<Option<f64>, _>("sample_size").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.2}", v) }),
        quantity_inspected: get_num(row, "quantity_inspected"),
        quantity_accepted: get_num(row, "quantity_accepted"),
        quantity_rejected: get_num(row, "quantity_rejected"),
        disposition: row.get("disposition"),
        rejection_reason: row.get("rejection_reason"),
        quality_score: row.try_get::<Option<f64>, _>("quality_score").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.2}", v) }),
        notes: row.get("notes"),
        status: row.get("status"),
        completed_at: row.get("completed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_detail(row: &sqlx::postgres::PgRow) -> InspectionDetail {
    InspectionDetail {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        inspection_id: row.get("inspection_id"),
        check_number: row.get("check_number"),
        check_name: row.get("check_name"),
        check_type: row.get("check_type"),
        specification: row.get("specification"),
        result: row.get("result"),
        measured_value: row.get("measured_value"),
        expected_value: row.get("expected_value"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
    }
}

fn row_to_delivery(row: &sqlx::postgres::PgRow) -> ReceiptDelivery {
    ReceiptDelivery {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        receipt_id: row.get("receipt_id"),
        receipt_line_id: row.get("receipt_line_id"),
        delivery_number: row.get("delivery_number"),
        subinventory: row.get("subinventory"),
        locator: row.get("locator"),
        quantity_delivered: get_num(row, "quantity_delivered"),
        uom: row.get("uom"),
        lot_number: row.get("lot_number"),
        serial_number: row.get("serial_number"),
        delivered_by: row.get("delivered_by"),
        delivered_by_name: row.get("delivered_by_name"),
        delivery_date: row.get("delivery_date"),
        destination_type: row.get("destination_type"),
        account_code: row.get("account_code"),
        notes: row.get("notes"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_return(row: &sqlx::postgres::PgRow) -> ReceiptReturn {
    ReceiptReturn {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        return_number: row.get("return_number"),
        receipt_id: row.get("receipt_id"),
        receipt_line_id: row.get("receipt_line_id"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        return_type: row.get("return_type"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        quantity_returned: get_num(row, "quantity_returned"),
        uom: row.get("uom"),
        unit_price: row.try_get::<Option<f64>, _>("unit_price").ok().flatten().map(|v| if v == v.floor() { format!("{:.0}", v) } else { format!("{:.2}", v) }),
        currency: row.get("currency"),
        return_reason: row.get("return_reason"),
        return_date: row.get("return_date"),
        carrier: row.get("carrier"),
        tracking_number: row.get("tracking_number"),
        credit_expected: row.get("credit_expected"),
        credit_memo_number: row.get("credit_memo_number"),
        status: row.get("status"),
        shipped_at: row.get("shipped_at"),
        credited_at: row.get("credited_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl ReceivingRepository for PostgresReceivingRepository {
    // ========================================================================
    // Locations
    // ========================================================================

    async fn create_location(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        location_type: &str, address: Option<&str>, city: Option<&str>,
        state: Option<&str>, country: Option<&str>, postal_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceivingLocation> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receiving_locations
                (organization_id, code, name, description, location_type,
                 address, city, state, country, postal_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, location_type = $5,
                    address = $6, city = $7, state = $8, country = $9,
                    postal_code = $10, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(location_type)
        .bind(address).bind(city).bind(state).bind(country).bind(postal_code)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_location(&row))
    }

    async fn get_location(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ReceivingLocation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.receiving_locations WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_location(&r)))
    }

    async fn list_locations(&self, org_id: Uuid) -> AtlasResult<Vec<ReceivingLocation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.receiving_locations WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_location).collect())
    }

    async fn delete_location(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.receiving_locations SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Receipts
    // ========================================================================

    async fn create_receipt(
        &self, org_id: Uuid, receipt_number: &str, receipt_type: &str,
        receipt_source: &str, supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        supplier_number: Option<&str>, purchase_order_id: Option<Uuid>,
        purchase_order_number: Option<&str>, receiving_location_id: Option<Uuid>,
        receiving_location_code: Option<&str>, receiving_date: chrono::NaiveDate,
        packing_slip_number: Option<&str>, bill_of_lading: Option<&str>,
        carrier: Option<&str>, tracking_number: Option<&str>,
        waybill_number: Option<&str>, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptHeader> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receipt_headers
                (organization_id, receipt_number, receipt_type, receipt_source,
                 supplier_id, supplier_name, supplier_number,
                 purchase_order_id, purchase_order_number,
                 receiving_location_id, receiving_location_code,
                 receiving_date, packing_slip_number, bill_of_lading,
                 carrier, tracking_number, waybill_number, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(receipt_number).bind(receipt_type).bind(receipt_source)
        .bind(supplier_id).bind(supplier_name).bind(supplier_number)
        .bind(purchase_order_id).bind(purchase_order_number)
        .bind(receiving_location_id).bind(receiving_location_code)
        .bind(receiving_date).bind(packing_slip_number).bind(bill_of_lading)
        .bind(carrier).bind(tracking_number).bind(waybill_number).bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("violates unique constraint") {
                AtlasError::Conflict(format!("Receipt number '{}' already exists", receipt_number))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;
        Ok(row_to_receipt(&row))
    }

    async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<ReceiptHeader>> {
        let row = sqlx::query("SELECT * FROM _atlas.receipt_headers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_receipt(&r)))
    }

    async fn get_receipt_by_number(&self, org_id: Uuid, receipt_number: &str) -> AtlasResult<Option<ReceiptHeader>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.receipt_headers WHERE organization_id = $1 AND receipt_number = $2"
        )
        .bind(org_id).bind(receipt_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_receipt(&r)))
    }

    async fn list_receipts(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptHeader>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.receipt_headers
            WHERE organization_id = $1
              AND ($2 IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR supplier_id = $3)
            ORDER BY receiving_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_receipt).collect())
    }

    async fn update_receipt_status(
        &self, id: Uuid, status: &str, received_by: Option<Uuid>, _closed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ReceiptHeader> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.receipt_headers
            SET status = $2,
                received_by = COALESCE($3, received_by),
                received_at = CASE WHEN $2 = 'received' AND received_at IS NULL THEN now() ELSE received_at END,
                closed_at = CASE WHEN $2 = 'closed' AND closed_at IS NULL THEN now() ELSE closed_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(received_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_receipt(&row))
    }

    // ========================================================================
    // Receipt Lines
    // ========================================================================

    async fn create_receipt_line(
        &self, org_id: Uuid, receipt_id: Uuid, line_number: i32,
        purchase_order_line_id: Option<Uuid>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        ordered_qty: &str, ordered_uom: Option<&str>,
        received_qty: &str, received_uom: Option<&str>,
        lot_number: Option<&str>, serial_numbers: serde_json::Value,
        expiration_date: Option<chrono::NaiveDate>, manufacture_date: Option<chrono::NaiveDate>,
        unit_price: Option<&str>, currency: Option<&str>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receipt_lines
                (organization_id, receipt_id, line_number,
                 purchase_order_line_id, item_id, item_code, item_description,
                 ordered_qty, ordered_uom, received_qty, received_uom,
                 lot_number, serial_numbers, expiration_date, manufacture_date,
                 unit_price, currency, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(receipt_id).bind(line_number)
        .bind(purchase_order_line_id).bind(item_id).bind(item_code).bind(item_description)
        .bind(ordered_qty.parse::<f64>().unwrap_or(0.0)).bind(ordered_uom)
        .bind(received_qty.parse::<f64>().unwrap_or(0.0)).bind(received_uom)
        .bind(lot_number).bind(serial_numbers).bind(expiration_date).bind(manufacture_date)
        .bind(unit_price.map(|v| v.parse::<f64>().unwrap_or(0.0))).bind(currency).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_line(&row))
    }

    async fn get_receipt_line(&self, id: Uuid) -> AtlasResult<Option<ReceiptLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.receipt_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_line(&r)))
    }

    async fn list_receipt_lines(&self, receipt_id: Uuid) -> AtlasResult<Vec<ReceiptLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.receipt_lines WHERE receipt_id = $1 ORDER BY line_number"
        )
        .bind(receipt_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_line).collect())
    }

    // ========================================================================
    // Inspections
    // ========================================================================

    async fn create_inspection(
        &self, org_id: Uuid, receipt_id: Uuid, receipt_line_id: Uuid,
        inspection_number: &str, inspection_template: Option<&str>,
        inspector_id: Option<Uuid>, inspector_name: Option<&str>,
        inspection_date: chrono::NaiveDate, sample_size: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptInspection> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receipt_inspections
                (organization_id, receipt_id, receipt_line_id, inspection_number,
                 inspection_template, inspector_id, inspector_name,
                 inspection_date, sample_size, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(receipt_id).bind(receipt_line_id).bind(inspection_number)
        .bind(inspection_template).bind(inspector_id).bind(inspector_name)
        .bind(inspection_date).bind(sample_size.map(|v| v.parse::<f64>().unwrap_or(0.0))).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_inspection(&row))
    }

    async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<ReceiptInspection>> {
        let row = sqlx::query("SELECT * FROM _atlas.receipt_inspections WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_inspection(&r)))
    }

    async fn list_inspections(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptInspection>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.receipt_inspections
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR receipt_id = $2)
            ORDER BY inspection_date DESC
            "#,
        )
        .bind(org_id).bind(receipt_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_inspection).collect())
    }

    async fn complete_inspection(
        &self, id: Uuid, quantity_inspected: &str, quantity_accepted: &str,
        quantity_rejected: &str, disposition: &str, quality_score: Option<&str>,
        rejection_reason: Option<&str>, notes: Option<&str>,
    ) -> AtlasResult<ReceiptInspection> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.receipt_inspections
            SET status = 'completed',
                quantity_inspected = $2,
                quantity_accepted = $3,
                quantity_rejected = $4,
                disposition = $5,
                quality_score = $6,
                rejection_reason = $7,
                notes = COALESCE($8, notes),
                completed_at = now(),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(quantity_inspected.parse::<f64>().unwrap_or(0.0)).bind(quantity_accepted.parse::<f64>().unwrap_or(0.0)).bind(quantity_rejected.parse::<f64>().unwrap_or(0.0))
        .bind(disposition).bind(quality_score.map(|v| v.parse::<f64>().unwrap_or(0.0))).bind(rejection_reason).bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_inspection(&row))
    }

    // ========================================================================
    // Inspection Details
    // ========================================================================

    async fn create_inspection_detail(
        &self, org_id: Uuid, inspection_id: Uuid, check_number: i32,
        check_name: &str, check_type: &str, specification: Option<&str>,
        result: &str, measured_value: Option<&str>, expected_value: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<InspectionDetail> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inspection_details
                (organization_id, inspection_id, check_number, check_name,
                 check_type, specification, result, measured_value, expected_value, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(inspection_id).bind(check_number).bind(check_name)
        .bind(check_type).bind(specification).bind(result)
        .bind(measured_value).bind(expected_value).bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_detail(&row))
    }

    async fn list_inspection_details(&self, inspection_id: Uuid) -> AtlasResult<Vec<InspectionDetail>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.inspection_details WHERE inspection_id = $1 ORDER BY check_number"
        )
        .bind(inspection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_detail).collect())
    }

    // ========================================================================
    // Deliveries
    // ========================================================================

    async fn create_delivery(
        &self, org_id: Uuid, receipt_id: Uuid, receipt_line_id: Uuid,
        delivery_number: &str, subinventory: Option<&str>, locator: Option<&str>,
        quantity_delivered: &str, uom: Option<&str>, lot_number: Option<&str>,
        serial_number: Option<&str>, delivered_by: Option<Uuid>,
        delivered_by_name: Option<&str>, destination_type: &str,
        account_code: Option<&str>, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptDelivery> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receipt_deliveries
                (organization_id, receipt_id, receipt_line_id, delivery_number,
                 subinventory, locator, quantity_delivered, uom,
                 lot_number, serial_number, delivered_by, delivered_by_name,
                 destination_type, account_code, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(receipt_id).bind(receipt_line_id).bind(delivery_number)
        .bind(subinventory).bind(locator).bind(quantity_delivered.parse::<f64>().unwrap_or(0.0)).bind(uom)
        .bind(lot_number).bind(serial_number).bind(delivered_by).bind(delivered_by_name)
        .bind(destination_type).bind(account_code).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_delivery(&row))
    }

    async fn get_delivery(&self, id: Uuid) -> AtlasResult<Option<ReceiptDelivery>> {
        let row = sqlx::query("SELECT * FROM _atlas.receipt_deliveries WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_delivery(&r)))
    }

    async fn list_deliveries(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptDelivery>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.receipt_deliveries
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR receipt_id = $2)
            ORDER BY delivery_date DESC
            "#,
        )
        .bind(org_id).bind(receipt_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_delivery).collect())
    }

    // ========================================================================
    // Returns
    // ========================================================================

    async fn create_return(
        &self, org_id: Uuid, return_number: &str, receipt_id: Option<Uuid>,
        receipt_line_id: Option<Uuid>, supplier_id: Option<Uuid>,
        supplier_name: Option<&str>, return_type: &str,
        item_id: Option<Uuid>, item_code: Option<&str>,
        item_description: Option<&str>, quantity_returned: &str,
        uom: Option<&str>, unit_price: Option<&str>, currency: Option<&str>,
        return_reason: Option<&str>, return_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptReturn> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.receipt_returns
                (organization_id, return_number, receipt_id, receipt_line_id,
                 supplier_id, supplier_name, return_type,
                 item_id, item_code, item_description,
                 quantity_returned, uom, unit_price, currency,
                 return_reason, return_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(return_number).bind(receipt_id).bind(receipt_line_id)
        .bind(supplier_id).bind(supplier_name).bind(return_type)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(quantity_returned.parse::<f64>().unwrap_or(0.0)).bind(uom).bind(unit_price.map(|v| v.parse::<f64>().unwrap_or(0.0))).bind(currency)
        .bind(return_reason).bind(return_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return(&row))
    }

    async fn get_return(&self, id: Uuid) -> AtlasResult<Option<ReceiptReturn>> {
        let row = sqlx::query("SELECT * FROM _atlas.receipt_returns WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_return(&r)))
    }

    async fn list_returns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptReturn>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.receipt_returns
            WHERE organization_id = $1 AND ($2 IS NULL OR status = $2)
            ORDER BY return_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_return).collect())
    }

    async fn update_return_status(
        &self, id: Uuid, status: &str, carrier: Option<&str>, tracking_number: Option<&str>,
    ) -> AtlasResult<ReceiptReturn> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.receipt_returns
            SET status = $2,
                carrier = COALESCE($3, carrier),
                tracking_number = COALESCE($4, tracking_number),
                shipped_at = CASE WHEN $2 = 'shipped' AND shipped_at IS NULL THEN now() ELSE shipped_at END,
                credited_at = CASE WHEN $2 = 'credited' AND credited_at IS NULL THEN now() ELSE credited_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(carrier).bind(tracking_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return(&row))
    }
}
