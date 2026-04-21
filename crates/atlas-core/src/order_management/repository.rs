//! Order Management Repository
//!
//! PostgreSQL storage for sales orders, order lines, holds, and shipments.

use atlas_shared::{
    SalesOrder, SalesOrderLine, OrderHold, FulfillmentShipment,
    OrderManagementDashboard, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for order management data storage
#[async_trait]
pub trait OrderManagementRepository: Send + Sync {
    // Sales Orders
    async fn create_order(
        &self, org_id: Uuid, order_number: &str, customer_id: Option<Uuid>,
        customer_name: Option<&str>, customer_po_number: Option<&str>,
        order_date: chrono::NaiveDate,
        requested_ship_date: Option<chrono::NaiveDate>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        ship_to_address: Option<&str>, bill_to_address: Option<&str>,
        currency_code: &str,
        payment_terms: Option<&str>, shipping_method: Option<&str>,
        sales_channel: Option<&str>,
        salesperson_id: Option<Uuid>, salesperson_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesOrder>;
    async fn get_order(&self, org_id: Uuid, order_number: &str) -> AtlasResult<Option<SalesOrder>>;
    async fn get_order_by_id(&self, id: Uuid) -> AtlasResult<Option<SalesOrder>>;
    async fn list_orders(&self, org_id: Uuid, status: Option<&str>, fulfillment_status: Option<&str>) -> AtlasResult<Vec<SalesOrder>>;
    async fn update_order_status(&self, id: Uuid, status: &str) -> AtlasResult<SalesOrder>;
    async fn update_order_fulfillment(&self, id: Uuid, fulfillment_status: &str) -> AtlasResult<SalesOrder>;
    async fn update_order_totals(&self, id: Uuid) -> AtlasResult<SalesOrder>;
    async fn update_order_dates(
        &self, id: Uuid,
        actual_ship_date: Option<chrono::NaiveDate>,
        actual_delivery_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<SalesOrder>;

    // Sales Order Lines
    async fn create_order_line(
        &self, org_id: Uuid, order_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
        quantity_ordered: &str, unit_selling_price: &str,
        unit_list_price: Option<&str>,
        discount_percent: Option<&str>, discount_amount: Option<&str>,
        tax_code: Option<&str>,
        requested_ship_date: Option<chrono::NaiveDate>,
        promised_delivery_date: Option<chrono::NaiveDate>,
        ship_from_warehouse: Option<&str>,
    ) -> AtlasResult<SalesOrderLine>;
    async fn get_order_line(&self, id: Uuid) -> AtlasResult<Option<SalesOrderLine>>;
    async fn list_order_lines(&self, order_id: Uuid) -> AtlasResult<Vec<SalesOrderLine>>;
    async fn update_line_quantities(
        &self, id: Uuid,
        quantity_shipped: Option<&str>,
        quantity_cancelled: Option<&str>,
        quantity_backordered: Option<&str>,
    ) -> AtlasResult<SalesOrderLine>;
    async fn update_line_status(&self, id: Uuid, status: &str, fulfillment_status: &str) -> AtlasResult<SalesOrderLine>;

    // Order Holds
    async fn create_hold(
        &self, org_id: Uuid, order_id: Uuid, order_line_id: Option<Uuid>,
        hold_type: &str, hold_reason: &str,
        applied_by: Option<Uuid>, applied_by_name: Option<&str>,
    ) -> AtlasResult<OrderHold>;
    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<OrderHold>>;
    async fn list_holds(&self, order_id: Uuid, active_only: bool) -> AtlasResult<Vec<OrderHold>>;
    async fn release_hold(
        &self, id: Uuid, released_by: Option<Uuid>, released_by_name: Option<&str>,
    ) -> AtlasResult<OrderHold>;

    // Fulfillment Shipments
    async fn create_shipment(
        &self, org_id: Uuid, shipment_number: &str, order_id: Uuid,
        order_line_ids: serde_json::Value,
        warehouse: Option<&str>, carrier: Option<&str>,
        shipping_method: Option<&str>,
        estimated_delivery_date: Option<chrono::NaiveDate>,
        shipped_by: Option<Uuid>, shipped_by_name: Option<&str>,
    ) -> AtlasResult<FulfillmentShipment>;
    async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<FulfillmentShipment>>;
    async fn list_shipments(&self, org_id: Uuid, status: Option<&str>, order_id: Option<Uuid>) -> AtlasResult<Vec<FulfillmentShipment>>;
    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<FulfillmentShipment>;
    async fn update_shipment_tracking(
        &self, id: Uuid, tracking_number: Option<&str>,
        actual_delivery_date: Option<chrono::NaiveDate>,
        delivery_confirmation: Option<&str>,
    ) -> AtlasResult<FulfillmentShipment>;
    async fn confirm_ship(&self, id: Uuid, ship_date: chrono::NaiveDate) -> AtlasResult<FulfillmentShipment>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<OrderManagementDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresOrderManagementRepository {
    pool: PgPool,
}

impl PostgresOrderManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_order(row: &sqlx::postgres::PgRow) -> SalesOrder {
    SalesOrder {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        order_number: row.get("order_number"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        customer_po_number: row.get("customer_po_number"),
        order_date: row.get("order_date"),
        requested_ship_date: row.get("requested_ship_date"),
        actual_ship_date: row.get("actual_ship_date"),
        requested_delivery_date: row.get("requested_delivery_date"),
        actual_delivery_date: row.get("actual_delivery_date"),
        ship_to_address: row.get("ship_to_address"),
        bill_to_address: row.get("bill_to_address"),
        currency_code: row.get("currency_code"),
        subtotal_amount: row_to_numeric(row, "subtotal_amount"),
        tax_amount: row_to_numeric(row, "tax_amount"),
        shipping_charges: row_to_numeric(row, "shipping_charges"),
        total_amount: row_to_numeric(row, "total_amount"),
        payment_terms: row.get("payment_terms"),
        shipping_method: row.get("shipping_method"),
        sales_channel: row.get("sales_channel"),
        salesperson_id: row.get("salesperson_id"),
        salesperson_name: row.get("salesperson_name"),
        status: row.get("status"),
        fulfillment_status: row.get("fulfillment_status"),
        submitted_at: row.get("submitted_at"),
        confirmed_at: row.get("confirmed_at"),
        closed_at: row.get("closed_at"),
        cancelled_at: row.get("cancelled_at"),
        cancellation_reason: row.get("cancellation_reason"),
        created_by: row.get("created_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_order_line(row: &sqlx::postgres::PgRow) -> SalesOrderLine {
    SalesOrderLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        order_id: row.get("order_id"),
        line_number: row.get("line_number"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        quantity_ordered: row_to_numeric(row, "quantity_ordered"),
        quantity_shipped: row_to_numeric(row, "quantity_shipped"),
        quantity_cancelled: row_to_numeric(row, "quantity_cancelled"),
        quantity_backordered: row_to_numeric(row, "quantity_backordered"),
        unit_selling_price: row_to_numeric(row, "unit_selling_price"),
        unit_list_price: row.try_get("unit_list_price").ok().map(|v: serde_json::Value| v.to_string()),
        line_amount: row_to_numeric(row, "line_amount"),
        discount_percent: row.try_get("discount_percent").ok().map(|v: serde_json::Value| v.to_string()),
        discount_amount: row.try_get("discount_amount").ok().map(|v: serde_json::Value| v.to_string()),
        tax_code: row.get("tax_code"),
        tax_amount: row_to_numeric(row, "tax_amount"),
        requested_ship_date: row.get("requested_ship_date"),
        actual_ship_date: row.get("actual_ship_date"),
        promised_delivery_date: row.get("promised_delivery_date"),
        ship_from_warehouse: row.get("ship_from_warehouse"),
        fulfillment_status: row.get("fulfillment_status"),
        status: row.get("status"),
        cancellation_reason: row.get("cancellation_reason"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_hold(row: &sqlx::postgres::PgRow) -> OrderHold {
    OrderHold {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        order_id: row.get("order_id"),
        order_line_id: row.get("order_line_id"),
        hold_type: row.get("hold_type"),
        hold_reason: row.get("hold_reason"),
        applied_by: row.get("applied_by"),
        applied_by_name: row.get("applied_by_name"),
        released_by: row.get("released_by"),
        released_by_name: row.get("released_by_name"),
        released_at: row.get("released_at"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_shipment(row: &sqlx::postgres::PgRow) -> FulfillmentShipment {
    FulfillmentShipment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        shipment_number: row.get("shipment_number"),
        order_id: row.get("order_id"),
        order_line_ids: row.try_get("order_line_ids").unwrap_or(serde_json::json!([])),
        warehouse: row.get("warehouse"),
        carrier: row.get("carrier"),
        tracking_number: row.get("tracking_number"),
        shipping_method: row.get("shipping_method"),
        ship_date: row.get("ship_date"),
        estimated_delivery_date: row.get("estimated_delivery_date"),
        actual_delivery_date: row.get("actual_delivery_date"),
        delivery_confirmation: row.get("delivery_confirmation"),
        status: row.get("status"),
        shipped_by: row.get("shipped_by"),
        shipped_by_name: row.get("shipped_by_name"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl OrderManagementRepository for PostgresOrderManagementRepository {
    // ========================================================================
    // Sales Orders
    // ========================================================================

    async fn create_order(
        &self, org_id: Uuid, order_number: &str, customer_id: Option<Uuid>,
        customer_name: Option<&str>, customer_po_number: Option<&str>,
        order_date: chrono::NaiveDate,
        requested_ship_date: Option<chrono::NaiveDate>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        ship_to_address: Option<&str>, bill_to_address: Option<&str>,
        currency_code: &str,
        payment_terms: Option<&str>, shipping_method: Option<&str>,
        sales_channel: Option<&str>,
        salesperson_id: Option<Uuid>, salesperson_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesOrder> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sales_orders
                (organization_id, order_number, customer_id, customer_name,
                 customer_po_number, order_date, requested_ship_date,
                 requested_delivery_date, ship_to_address, bill_to_address,
                 currency_code, payment_terms, shipping_method, sales_channel,
                 salesperson_id, salesperson_name, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(order_number).bind(customer_id).bind(customer_name)
        .bind(customer_po_number).bind(order_date).bind(requested_ship_date)
        .bind(requested_delivery_date).bind(ship_to_address).bind(bill_to_address)
        .bind(currency_code).bind(payment_terms).bind(shipping_method).bind(sales_channel)
        .bind(salesperson_id).bind(salesperson_name).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order(&row))
    }

    async fn get_order(&self, org_id: Uuid, order_number: &str) -> AtlasResult<Option<SalesOrder>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sales_orders WHERE organization_id=$1 AND order_number=$2"
        )
        .bind(org_id).bind(order_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_order(&r)))
    }

    async fn get_order_by_id(&self, id: Uuid) -> AtlasResult<Option<SalesOrder>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_orders WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_order(&r)))
    }

    async fn list_orders(&self, org_id: Uuid, status: Option<&str>, fulfillment_status: Option<&str>) -> AtlasResult<Vec<SalesOrder>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.sales_orders
            WHERE organization_id=$1
            AND ($2::text IS NULL OR status=$2)
            AND ($3::text IS NULL OR fulfillment_status=$3)
            ORDER BY order_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(fulfillment_status)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_order).collect())
    }

    async fn update_order_status(&self, id: Uuid, status: &str) -> AtlasResult<SalesOrder> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_orders SET status=$2,
                submitted_at=CASE WHEN $2='submitted' AND submitted_at IS NULL THEN now() ELSE submitted_at END,
                confirmed_at=CASE WHEN $2='confirmed' AND confirmed_at IS NULL THEN now() ELSE confirmed_at END,
                closed_at=CASE WHEN $2='closed' AND closed_at IS NULL THEN now() ELSE closed_at END,
                cancelled_at=CASE WHEN $2='cancelled' AND cancelled_at IS NULL THEN now() ELSE cancelled_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order(&row))
    }

    async fn update_order_fulfillment(&self, id: Uuid, fulfillment_status: &str) -> AtlasResult<SalesOrder> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_orders SET fulfillment_status=$2, updated_at=now() WHERE id=$1 RETURNING *",
        )
        .bind(id).bind(fulfillment_status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order(&row))
    }

    async fn update_order_totals(&self, id: Uuid) -> AtlasResult<SalesOrder> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_orders o SET
                subtotal_amount = COALESCE((SELECT SUM(line_amount) FROM _atlas.sales_order_lines WHERE order_id=$1), 0),
                tax_amount = COALESCE((SELECT SUM(tax_amount) FROM _atlas.sales_order_lines WHERE order_id=$1), 0),
                total_amount = COALESCE(o.shipping_charges, 0)
                    + COALESCE((SELECT SUM(line_amount) FROM _atlas.sales_order_lines WHERE order_id=$1), 0)
                    + COALESCE((SELECT SUM(tax_amount) FROM _atlas.sales_order_lines WHERE order_id=$1), 0),
                updated_at = now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order(&row))
    }

    async fn update_order_dates(
        &self, id: Uuid,
        actual_ship_date: Option<chrono::NaiveDate>,
        actual_delivery_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<SalesOrder> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_orders SET
                actual_ship_date=COALESCE($2, actual_ship_date),
                actual_delivery_date=COALESCE($3, actual_delivery_date),
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(actual_ship_date).bind(actual_delivery_date)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order(&row))
    }

    // ========================================================================
    // Sales Order Lines
    // ========================================================================

    async fn create_order_line(
        &self, org_id: Uuid, order_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
        quantity_ordered: &str, unit_selling_price: &str,
        unit_list_price: Option<&str>,
        discount_percent: Option<&str>, discount_amount: Option<&str>,
        tax_code: Option<&str>,
        requested_ship_date: Option<chrono::NaiveDate>,
        promised_delivery_date: Option<chrono::NaiveDate>,
        ship_from_warehouse: Option<&str>,
    ) -> AtlasResult<SalesOrderLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sales_order_lines
                (organization_id, order_id, line_number, item_id, item_code,
                 item_description, quantity_ordered, unit_selling_price,
                 unit_list_price, discount_percent, discount_amount,
                 tax_code, requested_ship_date, promised_delivery_date,
                 ship_from_warehouse)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8::numeric,$9::numeric,$10::numeric,$11::numeric,$12,$13,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(order_id).bind(line_number)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(quantity_ordered).bind(unit_selling_price)
        .bind(unit_list_price).bind(discount_percent).bind(discount_amount)
        .bind(tax_code).bind(requested_ship_date).bind(promised_delivery_date)
        .bind(ship_from_warehouse)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order_line(&row))
    }

    async fn get_order_line(&self, id: Uuid) -> AtlasResult<Option<SalesOrderLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.sales_order_lines WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_order_line(&r)))
    }

    async fn list_order_lines(&self, order_id: Uuid) -> AtlasResult<Vec<SalesOrderLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sales_order_lines WHERE order_id=$1 ORDER BY line_number"
        )
        .bind(order_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_order_line).collect())
    }

    async fn update_line_quantities(
        &self, id: Uuid,
        quantity_shipped: Option<&str>,
        quantity_cancelled: Option<&str>,
        quantity_backordered: Option<&str>,
    ) -> AtlasResult<SalesOrderLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sales_order_lines SET
                quantity_shipped = CASE WHEN $2::numeric IS NOT NULL THEN $2::numeric ELSE quantity_shipped END,
                quantity_cancelled = CASE WHEN $3::numeric IS NOT NULL THEN $3::numeric ELSE quantity_cancelled END,
                quantity_backordered = CASE WHEN $4::numeric IS NOT NULL THEN $4::numeric ELSE quantity_backordered END,
                updated_at = now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(quantity_shipped).bind(quantity_cancelled).bind(quantity_backordered)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order_line(&row))
    }

    async fn update_line_status(&self, id: Uuid, status: &str, fulfillment_status: &str) -> AtlasResult<SalesOrderLine> {
        let row = sqlx::query(
            "UPDATE _atlas.sales_order_lines SET status=$2, fulfillment_status=$3, updated_at=now() WHERE id=$1 RETURNING *",
        )
        .bind(id).bind(status).bind(fulfillment_status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_order_line(&row))
    }

    // ========================================================================
    // Order Holds
    // ========================================================================

    async fn create_hold(
        &self, org_id: Uuid, order_id: Uuid, order_line_id: Option<Uuid>,
        hold_type: &str, hold_reason: &str,
        applied_by: Option<Uuid>, applied_by_name: Option<&str>,
    ) -> AtlasResult<OrderHold> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.order_holds
                (organization_id, order_id, order_line_id, hold_type, hold_reason,
                 applied_by, applied_by_name)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *"#,
        )
        .bind(org_id).bind(order_id).bind(order_line_id)
        .bind(hold_type).bind(hold_reason)
        .bind(applied_by).bind(applied_by_name)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<OrderHold>> {
        let row = sqlx::query("SELECT * FROM _atlas.order_holds WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_hold(&r)))
    }

    async fn list_holds(&self, order_id: Uuid, active_only: bool) -> AtlasResult<Vec<OrderHold>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.order_holds WHERE order_id=$1 AND is_active=true ORDER BY created_at DESC"
            ).bind(order_id).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.order_holds WHERE order_id=$1 ORDER BY created_at DESC"
            ).bind(order_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_hold).collect())
    }

    async fn release_hold(
        &self, id: Uuid, released_by: Option<Uuid>, released_by_name: Option<&str>,
    ) -> AtlasResult<OrderHold> {
        let row = sqlx::query(
            r#"UPDATE _atlas.order_holds SET
                is_active=false, released_by=$2, released_by_name=$3,
                released_at=now(), updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(released_by).bind(released_by_name)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    // ========================================================================
    // Fulfillment Shipments
    // ========================================================================

    async fn create_shipment(
        &self, org_id: Uuid, shipment_number: &str, order_id: Uuid,
        order_line_ids: serde_json::Value,
        warehouse: Option<&str>, carrier: Option<&str>,
        shipping_method: Option<&str>,
        estimated_delivery_date: Option<chrono::NaiveDate>,
        shipped_by: Option<Uuid>, shipped_by_name: Option<&str>,
    ) -> AtlasResult<FulfillmentShipment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.fulfillment_shipments
                (organization_id, shipment_number, order_id, order_line_ids,
                 warehouse, carrier, shipping_method, estimated_delivery_date,
                 shipped_by, shipped_by_name)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *"#,
        )
        .bind(org_id).bind(shipment_number).bind(order_id).bind(order_line_ids)
        .bind(warehouse).bind(carrier).bind(shipping_method)
        .bind(estimated_delivery_date)
        .bind(shipped_by).bind(shipped_by_name)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_shipment(&row))
    }

    async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<FulfillmentShipment>> {
        let row = sqlx::query("SELECT * FROM _atlas.fulfillment_shipments WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_shipment(&r)))
    }

    async fn list_shipments(&self, org_id: Uuid, status: Option<&str>, order_id: Option<Uuid>) -> AtlasResult<Vec<FulfillmentShipment>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.fulfillment_shipments
            WHERE organization_id=$1
            AND ($2::text IS NULL OR status=$2)
            AND ($3::uuid IS NULL OR order_id=$3)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(order_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_shipment).collect())
    }

    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<FulfillmentShipment> {
        let row = sqlx::query(
            "UPDATE _atlas.fulfillment_shipments SET status=$2, updated_at=now() WHERE id=$1 RETURNING *",
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_shipment(&row))
    }

    async fn update_shipment_tracking(
        &self, id: Uuid, tracking_number: Option<&str>,
        actual_delivery_date: Option<chrono::NaiveDate>,
        delivery_confirmation: Option<&str>,
    ) -> AtlasResult<FulfillmentShipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.fulfillment_shipments SET
                tracking_number=COALESCE($2, tracking_number),
                actual_delivery_date=COALESCE($3, actual_delivery_date),
                delivery_confirmation=COALESCE($4, delivery_confirmation),
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(tracking_number).bind(actual_delivery_date).bind(delivery_confirmation)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_shipment(&row))
    }

    async fn confirm_ship(&self, id: Uuid, ship_date: chrono::NaiveDate) -> AtlasResult<FulfillmentShipment> {
        let row = sqlx::query(
            "UPDATE _atlas.fulfillment_shipments SET status='shipped', ship_date=$2, updated_at=now() WHERE id=$1 RETURNING *",
        )
        .bind(id).bind(ship_date)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_shipment(&row))
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<OrderManagementDashboard> {
        let row = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM _atlas.sales_orders WHERE organization_id=$1) as total_orders,
                (SELECT COUNT(*) FROM _atlas.sales_orders WHERE organization_id=$1 AND status IN ('draft','submitted','confirmed')) as open_orders,
                (SELECT COUNT(*) FROM _atlas.sales_orders WHERE organization_id=$1 AND fulfillment_status='in_process') as orders_in_fulfillment,
                (SELECT COUNT(*) FROM _atlas.sales_orders WHERE organization_id=$1 AND status='closed') as completed_orders,
                (SELECT COUNT(*) FROM _atlas.sales_orders WHERE organization_id=$1 AND status='cancelled') as cancelled_orders,
                (SELECT COALESCE(SUM(total_amount),0) FROM _atlas.sales_orders WHERE organization_id=$1 AND status != 'cancelled') as total_order_value,
                (SELECT COUNT(*) FROM _atlas.sales_orders o JOIN _atlas.order_holds h ON h.order_id=o.id WHERE o.organization_id=$1 AND h.is_active=true) as orders_on_hold,
                (SELECT COUNT(*) FROM _atlas.sales_order_lines WHERE organization_id=$1 AND quantity_backordered > 0) as backordered_lines,
                (SELECT COUNT(*) FROM _atlas.sales_orders WHERE organization_id=$1 AND requested_ship_date < now()::date AND actual_ship_date IS NULL AND status NOT IN ('cancelled','closed')) as overdue_shipments"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let total_orders: i64 = row.try_get("total_orders").unwrap_or(0);
        let open_orders: i64 = row.try_get("open_orders").unwrap_or(0);
        let orders_in_fulfillment: i64 = row.try_get("orders_in_fulfillment").unwrap_or(0);
        let completed_orders: i64 = row.try_get("completed_orders").unwrap_or(0);
        let cancelled_orders: i64 = row.try_get("cancelled_orders").unwrap_or(0);
        let total_order_value: serde_json::Value = row.try_get("total_order_value").unwrap_or(serde_json::json!("0"));
        let orders_on_hold: i64 = row.try_get("orders_on_hold").unwrap_or(0);
        let backordered_lines: i64 = row.try_get("backordered_lines").unwrap_or(0);
        let overdue_shipments: i64 = row.try_get("overdue_shipments").unwrap_or(0);

        let avg_order_value = if total_orders > 0 {
            let val: f64 = total_order_value.to_string().parse().unwrap_or(0.0);
            format!("{:.2}", val / total_orders as f64)
        } else {
            "0.00".to_string()
        };

        let fulfillment_rate = if total_orders > 0 {
            (completed_orders as f64 / total_orders as f64) * 100.0
        } else {
            0.0
        };

        let on_time = total_orders - overdue_shipments;
        let on_time_pct = if total_orders > 0 {
            (on_time as f64 / total_orders as f64) * 100.0
        } else {
            100.0
        };

        Ok(OrderManagementDashboard {
            total_orders: total_orders as i32,
            open_orders: open_orders as i32,
            orders_in_fulfillment: orders_in_fulfillment as i32,
            completed_orders: completed_orders as i32,
            cancelled_orders: cancelled_orders as i32,
            total_order_value: total_order_value.to_string(),
            average_order_value: avg_order_value,
            orders_on_hold: orders_on_hold as i32,
            backordered_lines: backordered_lines as i32,
            overdue_shipments: overdue_shipments as i32,
            orders_by_status: serde_json::json!({}),
            orders_by_channel: serde_json::json!({}),
            fulfillment_rate_pct: format!("{:.1}", fulfillment_rate),
            on_time_shipment_pct: format!("{:.1}", on_time_pct),
        })
    }
}
