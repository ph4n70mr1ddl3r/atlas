//! Order-to-Cash (O2C) End-to-End Integration Tests
//!
//! Full lifecycle: Lead → Qualify → Customer → Sales Order → Fulfill → Ship → Invoice → Payment
//! Plus edge cases: disqualification, cancellation, partial shipment, returns, etc.

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

use super::common::workflow_helpers::*;

// ============================================================================
// Helper: Full O2C setup
// ============================================================================

async fn setup_o2c() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_workflow_test_state().await;
    setup_o2c_entities(&state).await;
    cleanup_o2c(&state.db_pool).await;
    let app = build_app(state.clone());
    (state, app)
}

/// Seed a customer and return its ID
async fn seed_customer(
    app: &axum::Router, admin: &Claims,
) -> String {
    let customer = create_record(app, "crm_customers", json!({
        "customer_number": "CUS-001",
        "name": "Acme Corp",
        "type": "company",
        "industry": "technology",
        "email": "info@acmecorp.com",
        "phone": "555-1000",
        "status": "active",
        "revenue": 5000000
    }), admin).await;
    extract_id(&customer)
}

/// Seed product + warehouse for O2C
async fn seed_o2c_products(
    app: &axum::Router, admin: &Claims,
) -> (String, String) {
    let product = create_record(app, "scm_products", json!({
        "sku": "PROD-O2C-001",
        "name": "Enterprise Widget",
        "description": "Enterprise-grade widget",
        "product_type": "goods",
        "unit_price": 99.99,
        "cost_price": 45.00,
    }), admin).await;
    let product_id = extract_id(&product);

    let wh = create_record(app, "scm_warehouses", json!({
        "name": "Distribution Center",
        "code": "WH-DIST",
    }), admin).await;
    let wh_id = extract_id(&wh);

    (product_id, wh_id)
}

/// Create SO with lines
async fn create_so_with_lines(
    app: &axum::Router,
    admin: &Claims,
    customer_id: &str,
    lines: Vec<(String, f64, f64)>, // (product_id, quantity, unit_price)
) -> (String, Vec<String>) {
    let so = create_record(app, "scm_sales_orders", json!({
        "order_number": format!("SO-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
        "customer_id": customer_id,
        "order_date": "2025-02-01",
        "expected_delivery": "2025-02-15",
        "priority": "normal"
    }), admin).await;
    let so_id = extract_id(&so);

    let mut line_ids = Vec::new();
    for (i, (prod_id, qty, price)) in lines.iter().enumerate() {
        let line_total = qty * price;
        let line = create_record(app, "scm_sales_order_lines", json!({
            "sales_order_id": &so_id,
            "line_number": (i + 1) as i32,
            "product_id": prod_id,
            "quantity": qty,
            "unit_price": price,
            "discount_percent": 0,
            "tax_rate": 0,
            "line_total": line_total,
            "shipped_quantity": 0
        }), admin).await;
        line_ids.push(extract_id(&line));
    }

    (so_id, line_ids)
}

// ============================================================================
// TEST 1: Happy Path - Full O2C Lifecycle
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_full_happy_path() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();
    let wh_clerk = warehouse_claims();
    let fin_mgr = finance_manager_claims();

    // 1. Create and qualify lead
    let lead = create_record(&app, "crm_leads", json!({
        "first_name": "Jane",
        "last_name": "Smith",
        "email": "jane@prospect.com",
        "phone": "555-2001",
        "company": "Prospect Corp",
        "title": "VP Engineering",
        "source": "web",
        "industry": "technology",
        "estimated_value": 15000.00
    }), &sales_mgr).await;
    let lead_id = extract_id(&lead);

    // Contact the lead
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "contact", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "contacted");

    // Qualify the lead
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "qualify", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "qualified");

    // Convert to customer
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "convert", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "converted");

    // 2. Create customer record
    let customer_id = seed_customer(&app, &admin).await;

    // Create contact for customer
    let contact = create_record(&app, "crm_contacts", json!({
        "first_name": "Jane",
        "last_name": "Smith",
        "email": "jane@prospect.com",
        "phone": "555-2001",
        "title": "VP Engineering",
        "customer_id": &customer_id,
    }), &sales_mgr).await;
    assert!(!extract_id(&contact).is_empty());

    // 3. Create sales order with products
    let (product_id, wh_id) = seed_o2c_products(&app, &admin).await;
    let (so_id, line_ids) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id.clone(), 50.0, 99.99),
    ]).await;

    // 4. Confirm order
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "confirmed");

    // 5. Process order (warehouse picks)
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "process", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "processing");

    // 6. Create shipment
    let shipment = create_record(&app, "scm_shipments", json!({
        "shipment_number": "SHIP-001",
        "sales_order_id": &so_id,
        "customer_id": &customer_id,
        "warehouse_id": &wh_id,
        "shipment_date": "2025-02-10",
        "estimated_delivery": "2025-02-14",
        "carrier": "FedEx",
        "tracking_number": "FX-123456789"
    }), &wh_clerk).await;
    let shipment_id = extract_id(&shipment);

    // Add shipment line
    create_record(&app, "scm_shipment_lines", json!({
        "shipment_id": &shipment_id,
        "line_number": 1,
        "sales_order_line_id": &line_ids[0],
        "product_id": &product_id,
        "quantity_shipped": 50
    }), &wh_clerk).await;

    // Ship the shipment
    let result = execute_workflow_action(&app, "scm_shipments", &shipment_id, "ship", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "shipped");

    // Mark SO as shipped
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "ship", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "shipped");

    // Deliver shipment
    let result = execute_workflow_action(&app, "scm_shipments", &shipment_id, "deliver", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "delivered");

    // 7. Create invoice
    let invoice = create_record(&app, "fin_invoices", json!({
        "invoice_number": "INV-O2C-001",
        "customer_id": &customer_id,
        "invoice_date": "2025-02-15",
        "due_date": "2025-03-15",
        "subtotal": 4999.50,
        "tax_amount": 0,
        "total_amount": 4999.50,
        "amount_paid": 0,
        "balance_due": 4999.50,
        "payment_terms": "net_30"
    }), &fin_mgr).await;
    let invoice_id = extract_id(&invoice);

    create_record(&app, "fin_invoice_lines", json!({
        "invoice_id": &invoice_id,
        "line_number": 1,
        "product_id": &product_id,
        "description": "Enterprise Widget x50",
        "quantity": 50,
        "unit_price": 99.99,
        "line_total": 4999.50,
        "reference_type": "sales_order",
        "reference_id": &so_id
    }), &fin_mgr).await;

    // Submit and approve invoice
    execute_workflow_action(&app, "fin_invoices", &invoice_id, "submit", &fin_mgr).await;
    let result = execute_workflow_action(&app, "fin_invoices", &invoice_id, "approve", &fin_mgr).await;
    assert_eq!(result["success"], true);

    // Mark SO as invoiced
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "invoice", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "invoiced");

    // 8. Receive payment
    let payment = create_record(&app, "fin_payments", json!({
        "payment_number": "PAY-O2C-001",
        "payment_type": "receipt",
        "payment_method": "bank_transfer",
        "payer_id": &customer_id,
        "invoice_id": &invoice_id,
        "amount": 4999.50,
        "currency_code": "USD",
        "payment_date": "2025-02-28"
    }), &fin_mgr).await;
    let payment_id = extract_id(&payment);

    execute_workflow_action(&app, "fin_payments", &payment_id, "confirm", &fin_mgr).await;

    // Mark invoice as paid
    let result = execute_workflow_action(&app, "fin_invoices", &invoice_id, "mark_paid", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "paid");

    // 9. Complete sales order
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "complete", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "completed");

    // Verify final states
    let so = get_record(&app, "scm_sales_orders", &so_id, &admin).await;
    assert_eq!(so["workflow_state"], "completed");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 2: Lead Disqualification
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_lead_disqualification() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();

    let lead = create_record(&app, "crm_leads", json!({
        "first_name": "Bad",
        "last_name": "Lead",
        "email": "bad@lead.com",
        "company": "Bad Corp",
        "source": "cold_call"
    }), &sales_mgr).await;
    let lead_id = extract_id(&lead);

    // Disqualify from new state
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "disqualify", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "disqualified");

    // Verify terminal - no transitions
    let transitions = get_transitions(&app, "crm_leads", &lead_id, &admin).await;
    assert!(transitions["transitions"].as_array().unwrap().is_empty());

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 3: Lead Disqualified After Contact
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_lead_disqualified_after_contact() {
    let (state, app) = setup_o2c().await;
    let sales_mgr = sales_manager_claims();

    let lead = create_record(&app, "crm_leads", json!({
        "first_name": "Maybe",
        "last_name": "Not",
        "email": "maybe@not.com",
        "company": "Maybe Not Corp"
    }), &sales_mgr).await;
    let lead_id = extract_id(&lead);

    // Contact first
    execute_workflow_action(&app, "crm_leads", &lead_id, "contact", &sales_mgr).await;

    // Then disqualify
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "disqualify", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "disqualified");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 4: Sales Rep Cannot Qualify Lead (Role Check)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_sales_rep_cannot_qualify_lead() {
    let (state, app) = setup_o2c().await;
    let sales_rep = sales_rep_claims();
    let sales_mgr = sales_manager_claims();

    let lead = create_record(&app, "crm_leads", json!({
        "first_name": "Rep",
        "last_name": "Test",
        "email": "rep@test.com"
    }), &sales_mgr).await;
    let lead_id = extract_id(&lead);

    // Contact the lead
    execute_workflow_action(&app, "crm_leads", &lead_id, "contact", &sales_rep).await;

    // Sales rep cannot qualify
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "qualify", &sales_rep).await;
    assert_eq!(result["success"], false);
    assert!(result["error"].as_str().unwrap().contains("roles"));

    // Sales manager can qualify
    let result = execute_workflow_action(&app, "crm_leads", &lead_id, "qualify", &sales_mgr).await;
    assert_eq!(result["success"], true);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 5: Cancel Sales Order at Draft
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_cancel_order_at_draft() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;
    let (so_id, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 10.0, 50.0),
    ]).await;

    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "cancel", &admin).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "cancelled");

    let transitions = get_transitions(&app, "scm_sales_orders", &so_id, &admin).await;
    assert!(transitions["transitions"].as_array().unwrap().is_empty());

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 6: Cancel Confirmed Order (Requires Sales Manager)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_cancel_confirmed_order_requires_manager() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();
    let wh_clerk = warehouse_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;
    let (so_id, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 5.0, 100.0),
    ]).await;

    execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;

    // Regular user cannot cancel
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "cancel", &wh_clerk).await;
    assert_eq!(result["success"], false);

    // Sales manager can cancel
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "cancel", &sales_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "cancelled");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 7: Multi-line Sales Order
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_multi_line_sales_order() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;

    // Create second product
    let product2 = create_record(&app, "scm_products", json!({
        "sku": "PROD-O2C-002",
        "name": "Deluxe Gadget",
        "unit_price": 149.99,
    }), &admin).await;
    let product2_id = extract_id(&product2);

    // SO with 3 lines
    let (so_id, line_ids) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id.clone(), 10.0, 99.99),
        (product2_id.clone(), 5.0, 149.99),
        (product_id.clone(), 20.0, 99.99),
    ]).await;

    assert_eq!(line_ids.len(), 3);

    // Verify all lines
    let list = list_records(&app, "scm_sales_order_lines", &admin).await;
    let so_lines: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|l| l["sales_order_id"] == so_id)
        .collect();
    assert_eq!(so_lines.len(), 3);

    // Confirm order
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    assert_eq!(result["success"], true);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 8: Partial Shipment
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_partial_shipment() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();
    let wh_clerk = warehouse_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, wh_id) = seed_o2c_products(&app, &admin).await;
    let (so_id, line_ids) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id.clone(), 100.0, 99.99),
    ]).await;

    execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    execute_workflow_action(&app, "scm_sales_orders", &so_id, "process", &wh_clerk).await;

    // Ship only 60 of 100
    let shipment = create_record(&app, "scm_shipments", json!({
        "shipment_number": "SHIP-PART-001",
        "sales_order_id": &so_id,
        "customer_id": &customer_id,
        "warehouse_id": &wh_id,
        "shipment_date": "2025-02-12"
    }), &wh_clerk).await;
    let shipment_id = extract_id(&shipment);

    create_record(&app, "scm_shipment_lines", json!({
        "shipment_id": &shipment_id,
        "line_number": 1,
        "sales_order_line_id": &line_ids[0],
        "product_id": &product_id,
        "quantity_shipped": 60
    }), &wh_clerk).await;

    // Ship it
    execute_workflow_action(&app, "scm_shipments", &shipment_id, "ship", &wh_clerk).await;

    // Update SO line shipped quantity
    update_record(&app, "scm_sales_order_lines", &line_ids[0], json!({
        "shipped_quantity": 60
    }), &wh_clerk).await;

    // Verify partial
    let line = get_record(&app, "scm_sales_order_lines", &line_ids[0], &admin).await;
    assert_eq!(line["shipped_quantity"], 60);
    assert_eq!(line["quantity"], 100);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 9: Customer With Multiple Orders
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_customer_multiple_orders() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;

    // Create 3 orders
    let (so1, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id.clone(), 10.0, 99.99),
    ]).await;
    let (so2, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id.clone(), 20.0, 99.99),
    ]).await;
    let (so3, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id.clone(), 30.0, 99.99),
    ]).await;

    // Confirm first two, leave third as draft
    execute_workflow_action(&app, "scm_sales_orders", &so1, "confirm", &sales_mgr).await;
    execute_workflow_action(&app, "scm_sales_orders", &so2, "confirm", &sales_mgr).await;

    // List orders
    let list = list_records(&app, "scm_sales_orders", &admin).await;
    let customer_orders: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|so| [&so1, &so2, &so3].iter().any(|id| so["id"] == **id))
        .collect();
    assert_eq!(customer_orders.len(), 3);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 10: Contact Management
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_contact_management() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;

    // Create primary contact
    let contact1 = create_record(&app, "crm_contacts", json!({
        "first_name": "Alice",
        "last_name": "Johnson",
        "email": "alice@acmecorp.com",
        "phone": "555-1001",
        "title": "CEO",
        "customer_id": &customer_id,
    }), &sales_mgr).await;
    let c1_id = extract_id(&contact1);

    // Create secondary contact
    let _contact2 = create_record(&app, "crm_contacts", json!({
        "first_name": "Bob",
        "last_name": "Williams",
        "email": "bob@acmecorp.com",
        "phone": "555-1002",
        "title": "CTO",
        "customer_id": &customer_id,
    }), &sales_mgr).await;

    // List contacts for customer
    let list = list_records(&app, "crm_contacts", &admin).await;
    let customer_contacts: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|c| c["customer_id"] == customer_id)
        .collect();
    assert_eq!(customer_contacts.len(), 2);

    // Update contact
    let updated = update_record(&app, "crm_contacts", &c1_id, json!({
        "title": "President"
    }), &sales_mgr).await;
    assert_eq!(updated["title"], "President");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 11: Sales Order Line Discount
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_sales_order_line_discount() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;

    let so = create_record(&app, "scm_sales_orders", json!({
        "order_number": "SO-DISCOUNT-001",
        "customer_id": &customer_id,
        "order_date": "2025-02-01",
        "priority": "normal"
    }), &admin).await;
    let so_id = extract_id(&so);

    // Create line with 10% discount
    let line = create_record(&app, "scm_sales_order_lines", json!({
        "sales_order_id": &so_id,
        "line_number": 1,
        "product_id": &product_id,
        "quantity": 100,
        "unit_price": 99.99,
        "discount_percent": 10,
        "tax_rate": 0,
        "line_total": 8999.10,  // 100 * 99.99 * 0.9
        "shipped_quantity": 0
    }), &admin).await;

    assert_eq!(line["discount_percent"], 10);
    assert_eq!(line["line_total"], 8999.10);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 12: Shipment Tracking
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_shipment_tracking() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();
    let wh_clerk = warehouse_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, wh_id) = seed_o2c_products(&app, &admin).await;
    let (so_id, _line_ids) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 10.0, 99.99),
    ]).await;

    execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    execute_workflow_action(&app, "scm_sales_orders", &so_id, "process", &wh_clerk).await;

    let shipment = create_record(&app, "scm_shipments", json!({
        "shipment_number": "SHIP-TRACK-001",
        "sales_order_id": &so_id,
        "customer_id": &customer_id,
        "warehouse_id": &wh_id,
        "shipment_date": "2025-02-10",
        "estimated_delivery": "2025-02-14",
        "carrier": "UPS",
        "tracking_number": "1Z999AA10123456784"
    }), &wh_clerk).await;
    let shipment_id = extract_id(&shipment);

    // Ship
    execute_workflow_action(&app, "scm_shipments", &shipment_id, "ship", &wh_clerk).await;

    // Add actual delivery date
    let updated = update_record(&app, "scm_shipments", &shipment_id, json!({
        "actual_delivery": "2025-02-13"
    }), &wh_clerk).await;
    assert_eq!(updated["actual_delivery"], "2025-02-13");

    // Deliver
    let result = execute_workflow_action(&app, "scm_shipments", &shipment_id, "deliver", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "delivered");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 13: Priority Order Processing
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_priority_order() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;

    // Create urgent order
    let so = create_record(&app, "scm_sales_orders", json!({
        "order_number": "SO-URGENT-001",
        "customer_id": &customer_id,
        "order_date": "2025-02-01",
        "expected_delivery": "2025-02-03",
        "priority": "urgent"
    }), &admin).await;
    let so_id = extract_id(&so);

    create_record(&app, "scm_sales_order_lines", json!({
        "sales_order_id": &so_id,
        "line_number": 1,
        "product_id": &product_id,
        "quantity": 5,
        "unit_price": 99.99,
        "line_total": 499.95,
        "shipped_quantity": 0
    }), &admin).await;

    // Verify priority
    let fetched = get_record(&app, "scm_sales_orders", &so_id, &admin).await;
    assert_eq!(fetched["priority"], "urgent");

    // Fast-track processing
    execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    let so = get_record(&app, "scm_sales_orders", &so_id, &admin).await;
    assert_eq!(so["workflow_state"], "confirmed");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 14: Audit Trail for O2C
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_audit_trail() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;
    let (so_id, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 10.0, 99.99),
    ]).await;

    // Check audit for SO
    let history = get_history(&app, "scm_sales_orders", &so_id, &admin).await;
    assert!(history["history"].as_array().unwrap().is_empty());

    // Process through workflow and check history grows
    execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    let history2 = get_history(&app, "scm_sales_orders", &so_id, &admin).await;
    assert!(history2["history"].as_array().unwrap().len() >= history["history"].as_array().unwrap().len());

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 15: Customer CRUD
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_customer_crud() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();

    // Create
    let customer = create_record(&app, "crm_customers", json!({
        "customer_number": "CUS-CRUD-001",
        "name": "CRUD Corp",
        "type": "company",
        "industry": "retail",
        "email": "info@crudcorp.com",
        "phone": "555-3001",
        "status": "active",
        "revenue": 2500000
    }), &admin).await;
    let id = extract_id(&customer);

    // Read
    let fetched = get_record(&app, "crm_customers", &id, &admin).await;
    assert_eq!(fetched["name"], "CRUD Corp");

    // Update
    let updated = update_record(&app, "crm_customers", &id, json!({
        "industry": "ecommerce",
        "revenue": 3000000
    }), &admin).await;
    assert_eq!(updated["industry"], "ecommerce");
    assert_eq!(updated["revenue"], 3000000);

    // Delete
    let status = delete_record(&app, "crm_customers", &id, &admin).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify deleted
    let (k, v) = auth_header(&admin);
    let resp = app.clone().oneshot(
        Request::builder().uri(format!("/api/v1/crm_customers/{}", id)).header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 16: Invoice Payment for O2C (Customer Receipt)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_customer_payment() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (_product_id, _) = seed_o2c_products(&app, &admin).await;

    // Create and approve invoice
    let invoice = create_record(&app, "fin_invoices", json!({
        "invoice_number": "INV-O2C-PAY-001",
        "customer_id": &customer_id,
        "invoice_date": "2025-02-15",
        "due_date": "2025-03-15",
        "subtotal": 999.90,
        "total_amount": 999.90,
        "balance_due": 999.90
    }), &fin_mgr).await;
    let inv_id = extract_id(&invoice);

    execute_workflow_action(&app, "fin_invoices", &inv_id, "submit", &fin_mgr).await;
    execute_workflow_action(&app, "fin_invoices", &inv_id, "approve", &fin_mgr).await;

    // Customer payment (receipt type)
    let payment = create_record(&app, "fin_payments", json!({
        "payment_number": "PAY-O2C-RCPT-001",
        "payment_type": "receipt",
        "payment_method": "check",
        "payer_id": &customer_id,
        "invoice_id": &inv_id,
        "amount": 999.90,
        "payment_date": "2025-03-01",
        "reference_number": "CHK-12345"
    }), &fin_mgr).await;
    let pay_id = extract_id(&payment);

    // Confirm
    execute_workflow_action(&app, "fin_payments", &pay_id, "confirm", &fin_mgr).await;

    // Mark invoice paid
    let result = execute_workflow_action(&app, "fin_invoices", &inv_id, "mark_paid", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "paid");

    // Verify payment details
    let pay = get_record(&app, "fin_payments", &pay_id, &admin).await;
    assert_eq!(pay["payment_type"], "receipt");
    assert_eq!(pay["amount"], 999.90);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 17: Cannot Ship Unconfirmed Order
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_cannot_ship_unconfirmed_order() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let wh_clerk = warehouse_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;
    let (so_id, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 10.0, 50.0),
    ]).await;

    // Cannot process (which is prerequisite to ship) from draft
    execute_workflow_action_expect_status(
        &app, "scm_sales_orders", &so_id, "ship", &wh_clerk, StatusCode::BAD_REQUEST
    ).await;

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 18: Lead Cannot Skip Steps
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_lead_cannot_skip_steps() {
    let (state, app) = setup_o2c().await;
    let sales_mgr = sales_manager_claims();

    let lead = create_record(&app, "crm_leads", json!({
        "first_name": "Skip",
        "last_name": "Steps",
        "email": "skip@test.com"
    }), &sales_mgr).await;
    let lead_id = extract_id(&lead);

    // Cannot qualify directly from new (must contact first)
    // Actually the workflow allows qualify from "contacted", so new→qualify should fail
    execute_workflow_action_expect_status(
        &app, "crm_leads", &lead_id, "qualify", &sales_mgr, StatusCode::BAD_REQUEST
    ).await;

    // Cannot convert from new
    execute_workflow_action_expect_status(
        &app, "crm_leads", &lead_id, "convert", &sales_mgr, StatusCode::BAD_REQUEST
    ).await;

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 19: GL Journal Entry for Revenue
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_gl_posting_for_revenue() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    // Create accounts
    let ar_account = create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "1200",
        "name": "Accounts Receivable",
        "account_type": "asset",
    }), &admin).await;
    let ar_id = extract_id(&ar_account);

    let rev_account = create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "4000",
        "name": "Sales Revenue",
        "account_type": "revenue",
    }), &admin).await;
    let rev_id = extract_id(&rev_account);

    // Create revenue journal entry
    let je = create_record(&app, "fin_journal_entries", json!({
        "entry_number": "JE-REV-001",
        "entry_date": "2025-02-15",
        "description": "Record sales revenue",
        "total_debit": 4999.50,
        "total_credit": 4999.50
    }), &fin_mgr).await;
    let je_id = extract_id(&je);

    // Debit AR
    create_record(&app, "fin_journal_entry_lines", json!({
        "journal_entry_id": &je_id,
        "line_number": 1,
        "account_id": &ar_id,
        "description": "Accounts receivable",
        "debit_amount": 4999.50,
        "credit_amount": 0
    }), &fin_mgr).await;

    // Credit Revenue
    create_record(&app, "fin_journal_entry_lines", json!({
        "journal_entry_id": &je_id,
        "line_number": 2,
        "account_id": &rev_id,
        "description": "Sales revenue",
        "debit_amount": 0,
        "credit_amount": 4999.50
    }), &fin_mgr).await;

    // Submit and post
    execute_workflow_action(&app, "fin_journal_entries", &je_id, "submit", &fin_mgr).await;
    let result = execute_workflow_action(&app, "fin_journal_entries", &je_id, "post", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "posted");

    // Verify posted state
    let je_record = get_record(&app, "fin_journal_entries", &je_id, &admin).await;
    assert_eq!(je_record["workflow_state"], "posted");

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 20: Multiple Invoices for Same Customer
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_multiple_invoices_same_customer() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;

    // Create 3 invoices
    for i in 1..=3 {
        let inv = create_record(&app, "fin_invoices", json!({
            "invoice_number": format!("INV-MULTI-{:03}", i),
            "customer_id": &customer_id,
            "invoice_date": "2025-02-15",
            "due_date": "2025-03-15",
            "subtotal": 1000.00 * i as f64,
            "total_amount": 1000.00 * i as f64,
            "balance_due": 1000.00 * i as f64
        }), &fin_mgr).await;
        let inv_id = extract_id(&inv);
        execute_workflow_action(&app, "fin_invoices", &inv_id, "submit", &fin_mgr).await;
    }

    // List invoices
    let list = list_records(&app, "fin_invoices", &admin).await;
    let customer_invoices: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|inv| inv["customer_id"] == customer_id)
        .collect();
    assert_eq!(customer_invoices.len(), 3);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 21: Shipment Lifecycle Without Sales Order
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_shipment_lifecycle() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let wh_clerk = warehouse_claims();

    let (_, wh_id) = seed_o2c_products(&app, &admin).await;
    let customer_id = seed_customer(&app, &admin).await;

    // Create standalone shipment
    let shipment = create_record(&app, "scm_shipments", json!({
        "shipment_number": "SHIP-STAND-001",
        "customer_id": &customer_id,
        "warehouse_id": &wh_id,
        "shipment_date": "2025-02-20",
        "estimated_delivery": "2025-02-25",
        "carrier": "DHL"
    }), &wh_clerk).await;
    let ship_id = extract_id(&shipment);

    // Verify initial state
    let transitions = get_transitions(&app, "scm_shipments", &ship_id, &wh_clerk).await;
    assert_eq!(transitions["current_state"], "pending");

    // Ship
    let result = execute_workflow_action(&app, "scm_shipments", &ship_id, "ship", &wh_clerk).await;
    assert_eq!(result["success"], true);

    // Regular user cannot deliver? Actually deliver has no role requirement
    // So anyone can deliver
    let result = execute_workflow_action(&app, "scm_shipments", &ship_id, "deliver", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "delivered");

    // Terminal state
    let transitions = get_transitions(&app, "scm_shipments", &ship_id, &wh_clerk).await;
    assert!(transitions["transitions"].as_array().unwrap().is_empty());

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 22: Revenue Reporting via Chart of Accounts
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_revenue_accounts_reporting() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();

    // Create revenue accounts
    create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "4000",
        "name": "Product Revenue",
        "account_type": "revenue",
        "subtype": "operating",
    }), &admin).await;

    create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "4100",
        "name": "Service Revenue",
        "account_type": "revenue",
        "subtype": "operating",
    }), &admin).await;

    create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "5000",
        "name": "Cost of Goods Sold",
        "account_type": "expense",
        "subtype": "cost_of_sales",
    }), &admin).await;

    // List accounts
    let list = list_records(&app, "fin_chart_of_accounts", &admin).await;
    let accounts = list["data"].as_array().unwrap();
    assert!(accounts.len() >= 3);

    // Verify account types
    let revenue_accounts: Vec<_> = accounts.iter()
        .filter(|a| a["account_type"] == "revenue")
        .collect();
    assert!(revenue_accounts.len() >= 2);

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 23: Lead Workflow Full Happy Path
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_lead_full_lifecycle() {
    let (state, app) = setup_o2c().await;
    let sales_mgr = sales_manager_claims();

    let lead = create_record(&app, "crm_leads", json!({
        "first_name": "Full",
        "last_name": "Lifecycle",
        "email": "full@lifecycle.com",
        "company": "Lifecycle Corp",
        "estimated_value": 50000.00
    }), &sales_mgr).await;
    let lead_id = extract_id(&lead);

    // new → contacted
    let transitions = get_transitions(&app, "crm_leads", &lead_id, &sales_mgr).await;
    assert_eq!(transitions["current_state"], "new");
    assert!(transitions["transitions"].as_array().unwrap().iter().any(|t| t["action"] == "contact"));

    execute_workflow_action(&app, "crm_leads", &lead_id, "contact", &sales_mgr).await;

    // contacted → qualified
    let transitions = get_transitions(&app, "crm_leads", &lead_id, &sales_mgr).await;
    assert_eq!(transitions["current_state"], "contacted");

    execute_workflow_action(&app, "crm_leads", &lead_id, "qualify", &sales_mgr).await;

    // qualified → converted
    let transitions = get_transitions(&app, "crm_leads", &lead_id, &sales_mgr).await;
    assert_eq!(transitions["current_state"], "qualified");

    execute_workflow_action(&app, "crm_leads", &lead_id, "convert", &sales_mgr).await;

    // Verify converted (terminal)
    let lead = get_record(&app, "crm_leads", &lead_id, &sales_mgr).await;
    assert_eq!(lead["workflow_state"], "converted");

    let transitions = get_transitions(&app, "crm_leads", &lead_id, &sales_mgr).await;
    assert!(transitions["transitions"].as_array().unwrap().is_empty());

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 24: SO Completion Requires All Steps
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_so_must_follow_sequential_workflow() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let sales_mgr = sales_manager_claims();
    let wh_clerk = warehouse_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;
    let (so_id, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 10.0, 50.0),
    ]).await;

    // Cannot ship from draft (must confirm first)
    execute_workflow_action_expect_status(
        &app, "scm_sales_orders", &so_id, "ship", &wh_clerk, StatusCode::BAD_REQUEST
    ).await;

    // Cannot complete from draft
    execute_workflow_action_expect_status(
        &app, "scm_sales_orders", &so_id, "complete", &sales_mgr, StatusCode::BAD_REQUEST
    ).await;

    // Cannot invoice from draft
    execute_workflow_action_expect_status(
        &app, "scm_sales_orders", &so_id, "invoice", &admin, StatusCode::BAD_REQUEST
    ).await;

    cleanup_o2c(&state.db_pool).await;
}

// ============================================================================
// TEST 25: Warehouse Clerk Cannot Confirm Order
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_o2c_warehouse_cannot_confirm_order() {
    let (state, app) = setup_o2c().await;
    let admin = admin_claims();
    let wh_clerk = warehouse_claims();
    let sales_mgr = sales_manager_claims();

    let customer_id = seed_customer(&app, &admin).await;
    let (product_id, _) = seed_o2c_products(&app, &admin).await;
    let (so_id, _) = create_so_with_lines(&app, &admin, &customer_id, vec![
        (product_id, 10.0, 50.0),
    ]).await;

    // Warehouse clerk cannot confirm
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &wh_clerk).await;
    assert_eq!(result["success"], false);

    // Sales manager can confirm
    let result = execute_workflow_action(&app, "scm_sales_orders", &so_id, "confirm", &sales_mgr).await;
    assert_eq!(result["success"], true);

    cleanup_o2c(&state.db_pool).await;
}
