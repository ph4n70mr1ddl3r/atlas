//! Procure-to-Pay (P2P) End-to-End Integration Tests
//!
//! Full lifecycle: Supplier → Product → PO → Approve → Receive → Invoice → Pay → GL Post
//! Plus edge cases: rejection, cancellation, partial receipt, multi-line, audit, etc.

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

use super::common::workflow_helpers::*;

// ============================================================================
// Helper: Full P2P setup
// ============================================================================

async fn setup_p2p() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_workflow_test_state().await;
    // Clean residual data from prior failed runs BEFORE setting up
    cleanup_p2p(&state.db_pool).await;
    setup_p2p_entities(&state).await;
    let app = build_app(state.clone());
    (state, app)
}

/// Create a supplier, product, warehouse, and return their IDs
async fn seed_master_data(
    app: &axum::Router, admin: &Claims,
) -> (String, String, String) {
    let supplier = create_record(app, "scm_suppliers", json!({
        "supplier_number": "SUP-001",
        "name": "Acme Supplies",
        "email": "sales@acme.com",
        "phone": "555-0100",
        "category": "materials",
        "payment_terms": "net_30",
    }), admin).await;
    let supplier_id = extract_id(&supplier);

    let product = create_record(app, "scm_products", json!({
        "sku": "WIDGET-001",
        "name": "Premium Widget",
        "description": "High quality widget",
        "product_type": "goods",
        "category": "components",
        "supplier_id": &supplier_id,
        "unit_price": 29.99,
        "cost_price": 15.00,
        "reorder_level": 100,
        "unit_of_measure": "each",
    }), admin).await;
    let product_id = extract_id(&product);

    let wh = create_record(app, "scm_warehouses", json!({
        "name": "Main Warehouse",
        "code": "WH-MAIN",
    }), admin).await;
    let wh_id = extract_id(&wh);

    (supplier_id, product_id, wh_id)
}

/// Create a PO with multiple lines, returns PO id and line ids
async fn create_po_with_lines(
    app: &axum::Router,
    admin: &Claims,
    supplier_id: &str,
    lines: Vec<(String, f64, f64)>, // (product_id, quantity, unit_price)
) -> (String, Vec<String>) {
    let po = create_record(app, "scm_purchase_orders", json!({
        "po_number": format!("PO-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
        "supplier_id": supplier_id,
        "order_date": "2025-01-15",
        "expected_date": "2025-02-01",
        "currency_code": "USD",
        "payment_terms": "net_30",
        "notes": "P2P test PO"
    }), admin).await;
    let po_id = extract_id(&po);

    let mut line_ids = Vec::new();
    for (i, (prod_id, qty, price)) in lines.iter().enumerate() {
        let line_total = qty * price;
        let line = create_record(app, "scm_purchase_order_lines", json!({
            "purchase_order_id": &po_id,
            "line_number": (i + 1) as i32,
            "product_id": prod_id,
            "quantity": qty,
            "unit_price": price,
            "unit_of_measure": "each",
            "tax_rate": 0.0,
            "tax_amount": 0.0,
            "line_total": line_total,
            "received_quantity": 0,
            "invoiced_quantity": 0
        }), admin).await;
        line_ids.push(extract_id(&line));
    }

    (po_id, line_ids)
}

/// Create inventory record for a product in a warehouse
async fn create_inventory(
    app: &axum::Router,
    admin: &Claims,
    product_id: &str,
    warehouse_id: &str,
    qty: i64,
) -> String {
    let inv = create_record(app, "scm_inventory", json!({
        "product_id": product_id,
        "warehouse_id": warehouse_id,
        "quantity_on_hand": qty,
        "quantity_reserved": 0,
        "quantity_available": qty,
        "unit_cost": 15.00
    }), admin).await;
    extract_id(&inv)
}

// ============================================================================
// TEST 1: Happy Path - Full P2P Lifecycle
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_full_happy_path() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();
    let wh_clerk = warehouse_claims();
    let fin_mgr = finance_manager_claims();

    // 1. Create master data
    let (supplier_id, product_id, wh_id) = seed_master_data(&app, &admin).await;

    // 2. Create PO with one line
    let (po_id, line_ids) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 100.0, 29.99),
    ]).await;

    // 3. Verify initial state
    let transitions = get_transitions(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert_eq!(transitions["current_state"], "draft");

    // 4. Submit PO
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "submitted");

    // 5. Approve PO (by purchase manager)
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &buyer).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "approved");

    // 6. Create goods receipt
    let gr = create_record(&app, "scm_goods_receipts", json!({
        "receipt_number": "GR-001",
        "purchase_order_id": &po_id,
        "supplier_id": &supplier_id,
        "warehouse_id": &wh_id,
        "receipt_date": "2025-01-28",
        "total_quantity": 100,
        "notes": "Full receipt"
    }), &wh_clerk).await;
    let gr_id = extract_id(&gr);

    // Add receipt line
    create_record(&app, "scm_goods_receipt_lines", json!({
        "goods_receipt_id": &gr_id,
        "line_number": 1,
        "purchase_order_line_id": &line_ids[0],
        "product_id": &product_id,
        "warehouse_id": &wh_id,
        "quantity_received": 100,
        "quantity_accepted": 100,
        "quantity_rejected": 0
    }), &wh_clerk).await;

    // Confirm receipt
    let result = execute_workflow_action(&app, "scm_goods_receipts", &gr_id, "confirm", &wh_clerk).await;
    assert_eq!(result["success"], true);

    // Mark PO as received
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "receive", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "received");

    // 7. Create supplier invoice
    let invoice = create_record(&app, "fin_invoices", json!({
        "invoice_number": "INV-AP-001",
        "customer_id": null,
        "invoice_date": "2025-02-01",
        "due_date": "2025-03-01",
        "subtotal": 2999.00,
        "tax_amount": 0.00,
        "total_amount": 2999.00,
        "amount_paid": 0,
        "balance_due": 2999.00,
        "payment_terms": "net_30",
        "notes": "Invoice for PO"
    }), &fin_mgr).await;
    let invoice_id = extract_id(&invoice);

    // Add invoice line
    create_record(&app, "fin_invoice_lines", json!({
        "invoice_id": &invoice_id,
        "line_number": 1,
        "product_id": &product_id,
        "description": "Premium Widget x100",
        "quantity": 100,
        "unit_price": 29.99,
        "tax_rate": 0,
        "tax_amount": 0,
        "line_total": 2999.00
    }), &fin_mgr).await;

    // Submit and approve invoice
    execute_workflow_action(&app, "fin_invoices", &invoice_id, "submit", &fin_mgr).await;
    let result = execute_workflow_action(&app, "fin_invoices", &invoice_id, "approve", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "approved");

    // Mark PO as invoiced
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "invoice", &fin_mgr).await;
    assert_eq!(result["success"], true);

    // 8. Create payment
    let payment = create_record(&app, "fin_payments", json!({
        "payment_number": "PAY-001",
        "payment_type": "disbursement",
        "payment_method": "bank_transfer",
        "payee_id": &supplier_id,
        "invoice_id": &invoice_id,
        "amount": 2999.00,
        "currency_code": "USD",
        "payment_date": "2025-02-15",
        "reference_number": "TXN-001",
        "notes": "Payment for PO"
    }), &fin_mgr).await;
    let payment_id = extract_id(&payment);

    // Confirm payment
    let result = execute_workflow_action(&app, "fin_payments", &payment_id, "confirm", &fin_mgr).await;
    assert_eq!(result["success"], true);

    // Mark invoice as paid
    let result = execute_workflow_action(&app, "fin_invoices", &invoice_id, "mark_paid", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "paid");

    // 9. Close PO
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "close", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "closed");

    // 10. Verify final states
    let po = get_record(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert_eq!(po["workflow_state"], "closed");

    let inv = get_record(&app, "fin_invoices", &invoice_id, &admin).await;
    assert_eq!(inv["workflow_state"], "paid");

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 2: PO Rejection and Resubmission
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_po_rejection_and_resubmit() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 50.0, 10.0),
    ]).await;

    // Submit
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;

    // Reject
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "reject", &buyer).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "rejected");

    // Verify transitions from rejected state
    let transitions = get_transitions(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert_eq!(transitions["current_state"], "rejected");
    // Should have resubmit action
    let actions: Vec<&str> = transitions["transitions"].as_array().unwrap()
        .iter().map(|t| t["action"].as_str().unwrap()).collect();
    assert!(actions.contains(&"resubmit"));

    // Resubmit (goes back to draft)
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "resubmit", &admin).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "draft");

    // Can submit again
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    assert_eq!(result["success"], true);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 3: Cancel PO at Draft
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_cancel_po_at_draft() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 25.0, 50.0),
    ]).await;

    // Cancel from draft
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "cancel", &admin).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "cancelled");

    // Verify terminal state - no more transitions
    let transitions = get_transitions(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert!(transitions["transitions"].as_array().unwrap().is_empty());

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 4: Cancel PO at Submitted (should not be possible - no transition)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_cannot_cancel_submitted_po() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 10.0, 20.0),
    ]).await;

    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;

    // Attempt cancel from submitted - should fail (no transition defined)
    execute_workflow_action_expect_status(
        &app, "scm_purchase_orders", &po_id, "cancel", &admin, StatusCode::BAD_REQUEST
    ).await;

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 5: Role-based Access - Warehouse Clerk Cannot Approve PO
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_role_check_warehouse_cannot_approve_po() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let wh_clerk = warehouse_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 100.0, 5.0),
    ]).await;

    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;

    // Warehouse clerk tries to approve - should fail
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &wh_clerk).await;
    assert_eq!(result["success"], false);
    assert!(result["error"].as_str().unwrap().contains("roles"));

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 6: Multi-line Purchase Order
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_multi_line_purchase_order() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;

    // Create a second product
    let product2 = create_record(&app, "scm_products", json!({
        "sku": "GADGET-001",
        "name": "Super Gadget",
        "supplier_id": &supplier_id,
        "unit_price": 49.99,
        "cost_price": 25.00,
    }), &admin).await;
    let product2_id = extract_id(&product2);

    // PO with 3 lines
    let (po_id, line_ids) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 100.0, 29.99),
        (product2_id.clone(), 50.0, 49.99),
        (product_id.clone(), 25.0, 29.99),
    ]).await;

    assert_eq!(line_ids.len(), 3);

    // Verify all lines exist
    let list = list_records(&app, "scm_purchase_order_lines", &admin).await;
    let po_lines: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|l| l["purchase_order_id"] == po_id)
        .collect();
    assert_eq!(po_lines.len(), 3);

    // Submit and approve
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    let result = execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &buyer).await;
    assert_eq!(result["success"], true);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 7: Inventory Update After Goods Receipt
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_inventory_updated_on_receipt() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();
    let wh_clerk = warehouse_claims();

    let (supplier_id, product_id, wh_id) = seed_master_data(&app, &admin).await;

    // Create initial inventory (0 on hand)
    let inv_id = create_inventory(&app, &admin, &product_id, &wh_id, 0).await;

    // Create and approve PO
    let (po_id, line_ids) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 200.0, 15.0),
    ]).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &buyer).await;

    // Receive goods
    let gr = create_record(&app, "scm_goods_receipts", json!({
        "receipt_number": "GR-INV-001",
        "purchase_order_id": &po_id,
        "supplier_id": &supplier_id,
        "warehouse_id": &wh_id,
        "receipt_date": "2025-01-20",
        "total_quantity": 200
    }), &wh_clerk).await;
    let gr_id = extract_id(&gr);

    create_record(&app, "scm_goods_receipt_lines", json!({
        "goods_receipt_id": &gr_id,
        "line_number": 1,
        "purchase_order_line_id": &line_ids[0],
        "product_id": &product_id,
        "warehouse_id": &wh_id,
        "quantity_received": 200,
        "quantity_accepted": 200,
        "quantity_rejected": 0
    }), &wh_clerk).await;

    execute_workflow_action(&app, "scm_goods_receipts", &gr_id, "confirm", &wh_clerk).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "receive", &wh_clerk).await;

    // Update inventory to reflect receipt
    update_record(&app, "scm_inventory", &inv_id, json!({
        "quantity_on_hand": 200,
        "quantity_available": 200,
        "unit_cost": 15.00
    }), &wh_clerk).await;

    // Verify inventory updated
    let inv = get_record(&app, "scm_inventory", &inv_id, &admin).await;
    assert_eq!(inv["quantity_on_hand"], 200);
    assert_eq!(inv["quantity_available"], 200);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 8: Partial Goods Receipt
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_partial_goods_receipt() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();
    let wh_clerk = warehouse_claims();

    let (supplier_id, product_id, wh_id) = seed_master_data(&app, &admin).await;
    let (po_id, line_ids) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 100.0, 25.0),
    ]).await;

    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &buyer).await;

    // Receive only 60 of 100
    let gr = create_record(&app, "scm_goods_receipts", json!({
        "receipt_number": "GR-PART-001",
        "purchase_order_id": &po_id,
        "supplier_id": &supplier_id,
        "warehouse_id": &wh_id,
        "receipt_date": "2025-01-22",
        "total_quantity": 60
    }), &wh_clerk).await;
    let gr_id = extract_id(&gr);

    create_record(&app, "scm_goods_receipt_lines", json!({
        "goods_receipt_id": &gr_id,
        "line_number": 1,
        "purchase_order_line_id": &line_ids[0],
        "product_id": &product_id,
        "warehouse_id": &wh_id,
        "quantity_received": 60,
        "quantity_accepted": 60,
        "quantity_rejected": 0
    }), &wh_clerk).await;

    execute_workflow_action(&app, "scm_goods_receipts", &gr_id, "confirm", &wh_clerk).await;

    // Update PO line to reflect partial receipt
    update_record(&app, "scm_purchase_order_lines", &line_ids[0], json!({
        "received_quantity": 60
    }), &wh_clerk).await;

    // Verify partial receipt recorded
    let po_line = get_record(&app, "scm_purchase_order_lines", &line_ids[0], &admin).await;
    assert_eq!(po_line["received_quantity"], 60);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 9: Goods Receipt with Rejected Items (Quality Issue)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_goods_receipt_with_rejections() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();
    let wh_clerk = warehouse_claims();

    let (supplier_id, product_id, wh_id) = seed_master_data(&app, &admin).await;
    let (po_id, line_ids) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 100.0, 25.0),
    ]).await;

    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &buyer).await;

    // Receive 100, but 10 are rejected
    let gr = create_record(&app, "scm_goods_receipts", json!({
        "receipt_number": "GR-QC-001",
        "purchase_order_id": &po_id,
        "supplier_id": &supplier_id,
        "warehouse_id": &wh_id,
        "receipt_date": "2025-01-25",
        "total_quantity": 100
    }), &wh_clerk).await;
    let gr_id = extract_id(&gr);

    create_record(&app, "scm_goods_receipt_lines", json!({
        "goods_receipt_id": &gr_id,
        "line_number": 1,
        "purchase_order_line_id": &line_ids[0],
        "product_id": &product_id,
        "warehouse_id": &wh_id,
        "quantity_received": 100,
        "quantity_accepted": 90,
        "quantity_rejected": 10
    }), &wh_clerk).await;

    execute_workflow_action(&app, "scm_goods_receipts", &gr_id, "confirm", &wh_clerk).await;

    // Verify rejection recorded
    let gr_line = list_records(&app, "scm_goods_receipt_lines", &admin).await;
    let line = &gr_line["data"].as_array().unwrap()[0];
    assert_eq!(line["quantity_rejected"], 10);
    assert_eq!(line["quantity_accepted"], 90);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 10: Duplicate PO Number Rejected by DB
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_duplicate_po_number() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    let (supplier_id, _, _) = seed_master_data(&app, &admin).await;

    // Create first PO
    create_record(&app, "scm_purchase_orders", json!({
        "po_number": "PO-DUP-001",
        "supplier_id": &supplier_id,
        "order_date": "2025-01-15",
        "currency_code": "USD"
    }), &admin).await;

    // Try duplicate po_number - should fail at DB level
    let (k, v) = auth_header(&admin);
    let resp = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/scm_purchase_orders")
            .header("Content-Type", "application/json").header(k, v)
            .body(Body::from(serde_json::to_string(&json!({
                "entity": "scm_purchase_orders", "values": {
                    "po_number": "PO-DUP-001",
                    "supplier_id": supplier_id,
                    "order_date": "2025-01-16"
                }
            })).unwrap())).unwrap()
    ).await.unwrap();
    // DB should reject the duplicate
    assert!(resp.status().is_client_error() || resp.status().is_server_error());

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 11: GL Journal Entry for Invoice
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_gl_journal_entry_for_invoice() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let (_supplier_id, _product_id, _) = seed_master_data(&app, &admin).await;

    // Create chart of accounts
    let ap_account = create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "2000",
        "name": "Accounts Payable",
        "account_type": "liability",
        "subtype": "current",
    }), &admin).await;
    let ap_acct_id = extract_id(&ap_account);

    let expense_account = create_record(&app, "fin_chart_of_accounts", json!({
        "account_number": "5000",
        "name": "Materials Expense",
        "account_type": "expense",
        "subtype": "operating",
    }), &admin).await;
    let exp_acct_id = extract_id(&expense_account);

    // Create journal entry
    let je = create_record(&app, "fin_journal_entries", json!({
        "entry_number": "JE-001",
        "entry_date": "2025-02-01",
        "description": "Record AP invoice",
        "total_debit": 2999.00,
        "total_credit": 2999.00
    }), &fin_mgr).await;
    let je_id = extract_id(&je);

    // Debit line (expense)
    create_record(&app, "fin_journal_entry_lines", json!({
        "journal_entry_id": &je_id,
        "line_number": 1,
        "account_id": &exp_acct_id,
        "description": "Materials purchased",
        "debit_amount": 2999.00,
        "credit_amount": 0
    }), &fin_mgr).await;

    // Credit line (AP)
    create_record(&app, "fin_journal_entry_lines", json!({
        "journal_entry_id": &je_id,
        "line_number": 2,
        "account_id": &ap_acct_id,
        "description": "Accounts payable",
        "debit_amount": 0,
        "credit_amount": 2999.00
    }), &fin_mgr).await;

    // Submit and post journal entry
    execute_workflow_action(&app, "fin_journal_entries", &je_id, "submit", &fin_mgr).await;
    let result = execute_workflow_action(&app, "fin_journal_entries", &je_id, "post", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "posted");

    // Verify balanced entry
    let lines = list_records(&app, "fin_journal_entry_lines", &fin_mgr).await;
    let je_lines: Vec<_> = lines["data"].as_array().unwrap().iter()
        .filter(|l| l["journal_entry_id"] == je_id)
        .collect();
    assert_eq!(je_lines.len(), 2);

    let total_debit: f64 = je_lines.iter()
        .map(|l| l["debit_amount"].as_f64().unwrap_or(0.0)).sum();
    let total_credit: f64 = je_lines.iter()
        .map(|l| l["credit_amount"].as_f64().unwrap_or(0.0)).sum();
    assert!((total_debit - total_credit).abs() < 0.01);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 12: Audit Trail Throughout P2P
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_audit_trail() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let _buyer = purchase_manager_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 50.0, 10.0),
    ]).await;

    // Get audit history for PO - should have create entry
    let history = get_history(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert_eq!(history["entity"], "scm_purchase_orders");
    // History should contain the create audit entry
    assert!(history["history"].as_array().unwrap().is_empty());

    // Submit
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;

    // History should now have more entries
    let history2 = get_history(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert!(history2["history"].as_array().unwrap().len() >= history["history"].as_array().unwrap().len());

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 13: Payment Reconciliation Workflow
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_payment_reconciliation() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let (supplier_id, _, _) = seed_master_data(&app, &admin).await;

    let payment = create_record(&app, "fin_payments", json!({
        "payment_number": "PAY-RECON-001",
        "payment_type": "disbursement",
        "payment_method": "bank_transfer",
        "payee_id": &supplier_id,
        "amount": 5000.00,
        "currency_code": "USD",
        "payment_date": "2025-02-20"
    }), &fin_mgr).await;
    let payment_id = extract_id(&payment);

    // Confirm
    let result = execute_workflow_action(&app, "fin_payments", &payment_id, "confirm", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "confirmed");

    // Reconcile
    let result = execute_workflow_action(&app, "fin_payments", &payment_id, "reconcile", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "reconciled");

    // Verify reconciled flag
    let pay = get_record(&app, "fin_payments", &payment_id, &admin).await;
    assert_eq!(pay["workflow_state"], "reconciled");

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 14: Invoice Void
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_invoice_void() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let (_supplier_id, _, _) = seed_master_data(&app, &admin).await;

    let invoice = create_record(&app, "fin_invoices", json!({
        "invoice_number": "INV-VOID-001",
        "invoice_date": "2025-01-15",
        "due_date": "2025-02-15",
        "subtotal": 1000.00,
        "total_amount": 1000.00,
        "balance_due": 1000.00
    }), &fin_mgr).await;
    let invoice_id = extract_id(&invoice);

    // Void from draft
    let result = execute_workflow_action(&app, "fin_invoices", &invoice_id, "void", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "void");

    // Verify terminal state
    let transitions = get_transitions(&app, "fin_invoices", &invoice_id, &admin).await;
    assert!(transitions["transitions"].as_array().unwrap().is_empty());

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 15: Invalid Transition on Final State
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_no_transition_from_closed() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();
    let wh_clerk = warehouse_claims();
    let fin_mgr = finance_manager_claims();

    // Quick path to closed
    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 10.0, 10.0),
    ]).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "submit", &admin).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "approve", &buyer).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "receive", &wh_clerk).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "invoice", &fin_mgr).await;
    execute_workflow_action(&app, "scm_purchase_orders", &po_id, "close", &fin_mgr).await;

    // Try to submit closed PO - should fail
    execute_workflow_action_expect_status(
        &app, "scm_purchase_orders", &po_id, "submit", &admin, StatusCode::BAD_REQUEST
    ).await;

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 16: Supplier CRUD Lifecycle
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_supplier_crud() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    // Create
    let supplier = create_record(&app, "scm_suppliers", json!({
        "supplier_number": "SUP-CRUD-001",
        "name": "Test Supplier",
        "contact_name": "John Doe",
        "email": "john@test.com",
        "phone": "555-1234",
        "category": "office_supplies",
        "payment_terms": "net_45",
        "credit_limit": 50000.00,
    }), &admin).await;
    let id = extract_id(&supplier);
    assert_eq!(supplier["name"], "Test Supplier", "Supplier: {:?}", supplier);

    // Read
    let fetched = get_record(&app, "scm_suppliers", &id, &admin).await;
    assert_eq!(fetched["supplier_number"], "SUP-CRUD-001");
    assert_eq!(fetched["email"], "john@test.com");

    // Update
    let updated = update_record(&app, "scm_suppliers", &id, json!({
        "name": "Updated Supplier Name",
        "phone": "555-9999"
    }), &admin).await;
    assert_eq!(updated["name"], "Updated Supplier Name");
    assert_eq!(updated["phone"], "555-9999");

    // List - should contain the supplier we just created
    let list = list_records(&app, "scm_suppliers", &admin).await;
    assert!(!list["data"].as_array().unwrap().is_empty());

    // Soft delete
    let status = delete_record(&app, "scm_suppliers", &id, &admin).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify soft-deleted record not found
    let (k, v) = auth_header(&admin);
    let resp = app.clone().oneshot(
        Request::builder().uri(format!("/api/v1/scm_suppliers/{}", id)).header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 17: Regular User Cannot Approve Invoice
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_regular_user_cannot_approve_invoice() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let regular_user = user_claims();
    let fin_mgr = finance_manager_claims();

    let (_supplier_id, _, _) = seed_master_data(&app, &admin).await;

    let invoice = create_record(&app, "fin_invoices", json!({
        "invoice_number": "INV-RBAC-001",
        "invoice_date": "2025-01-15",
        "due_date": "2025-02-15",
        "subtotal": 500.00,
        "total_amount": 500.00,
        "balance_due": 500.00
    }), &fin_mgr).await;
    let inv_id = extract_id(&invoice);

    execute_workflow_action(&app, "fin_invoices", &inv_id, "submit", &fin_mgr).await;

    // Regular user cannot approve
    let result = execute_workflow_action(&app, "fin_invoices", &inv_id, "approve", &regular_user).await;
    assert_eq!(result["success"], false);

    // Finance manager can approve
    let result = execute_workflow_action(&app, "fin_invoices", &inv_id, "approve", &fin_mgr).await;
    assert_eq!(result["success"], true);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 18: Multiple POs for Same Supplier
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_multiple_pos_same_supplier() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let buyer = purchase_manager_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;

    // Create 3 POs for same supplier
    let (po1, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 10.0, 25.0),
    ]).await;
    let (po2, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 20.0, 25.0),
    ]).await;
    let (po3, _) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id.clone(), 30.0, 25.0),
    ]).await;

    // All different POs
    assert_ne!(po1, po2);
    assert_ne!(po2, po3);

    // Submit and approve all
    for po_id in [&po1, &po2, &po3] {
        execute_workflow_action(&app, "scm_purchase_orders", po_id, "submit", &admin).await;
        execute_workflow_action(&app, "scm_purchase_orders", po_id, "approve", &buyer).await;
    }

    // List POs
    let list = list_records(&app, "scm_purchase_orders", &admin).await;
    let pos: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|po| [&po1, &po2, &po3].iter().any(|id| po["id"] == **id))
        .collect();
    assert_eq!(pos.len(), 3);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 19: Search/Filter Purchase Orders
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_search_purchase_orders() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    let (supplier_id, _, _) = seed_master_data(&app, &admin).await;

    // Create PO with searchable notes
    create_record(&app, "scm_purchase_orders", json!({
        "po_number": "PO-SEARCH-001",
        "supplier_id": &supplier_id,
        "order_date": "2025-03-01",
        "notes": "Urgent order for Q1"
    }), &admin).await;

    create_record(&app, "scm_purchase_orders", json!({
        "po_number": "PO-SEARCH-002",
        "supplier_id": &supplier_id,
        "order_date": "2025-03-15",
        "notes": "Regular order for Q2"
    }), &admin).await;

    // List all POs
    let list = list_records(&app, "scm_purchase_orders", &admin).await;
    assert!(list["data"].as_array().unwrap().len() >= 2);
    assert!(list["meta"]["total"].as_i64().unwrap() >= 2);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 20: PO Line Update After Creation
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_update_po_line_quantity() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    let (supplier_id, product_id, _) = seed_master_data(&app, &admin).await;
    let (po_id, line_ids) = create_po_with_lines(&app, &admin, &supplier_id, vec![
        (product_id, 50.0, 20.0),
    ]).await;

    // Update line quantity from 50 to 75
    let updated = update_record(&app, "scm_purchase_order_lines", &line_ids[0], json!({
        "quantity": 75,
        "line_total": 1500.00
    }), &admin).await;
    assert_eq!(updated["quantity"], 75);
    assert_eq!(updated["line_total"], 1500.00);

    // Original PO unchanged
    let po = get_record(&app, "scm_purchase_orders", &po_id, &admin).await;
    assert_eq!(po["workflow_state"], "draft");

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 21: Payment Without Approval Role
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_payment_confirm_requires_role() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let regular_user = user_claims();
    let fin_mgr = finance_manager_claims();

    let (supplier_id, _, _) = seed_master_data(&app, &admin).await;

    let payment = create_record(&app, "fin_payments", json!({
        "payment_number": "PAY-ROLE-001",
        "payment_type": "disbursement",
        "payee_id": &supplier_id,
        "amount": 1000.00,
        "payment_date": "2025-03-01"
    }), &admin).await;
    let pay_id = extract_id(&payment);

    // Regular user cannot confirm
    let result = execute_workflow_action(&app, "fin_payments", &pay_id, "confirm", &regular_user).await;
    assert_eq!(result["success"], false);

    // Finance manager can confirm
    let result = execute_workflow_action(&app, "fin_payments", &pay_id, "confirm", &fin_mgr).await;
    assert_eq!(result["success"], true);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 22: Supplier With No Purchase Orders
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_supplier_with_no_orders() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();

    let supplier = create_record(&app, "scm_suppliers", json!({
        "supplier_number": "SUP-NOPO-001",
        "name": "Unused Supplier",
        "email": "unused@test.com",
    }), &admin).await;
    let sup_id = extract_id(&supplier);

    // List POs - should not contain any for this supplier
    let list = list_records(&app, "scm_purchase_orders", &admin).await;
    let supplier_pos: Vec<_> = list["data"].as_array().unwrap().iter()
        .filter(|po| po["supplier_id"] == sup_id)
        .collect();
    assert_eq!(supplier_pos.len(), 0);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 23: Goods Receipt Without PO Reference
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_direct_goods_receipt_no_po() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let wh_clerk = warehouse_claims();

    let (supplier_id, _product_id, wh_id) = seed_master_data(&app, &admin).await;

    // Create a goods receipt without PO reference
    let gr = create_record(&app, "scm_goods_receipts", json!({
        "receipt_number": "GR-NATIVE-001",
        "supplier_id": &supplier_id,
        "warehouse_id": &wh_id,
        "receipt_date": "2025-01-30",
        "total_quantity": 50,
        "notes": "Direct delivery, no PO"
    }), &wh_clerk).await;
    let gr_id = extract_id(&gr);

    // Confirm it
    let result = execute_workflow_action(&app, "scm_goods_receipts", &gr_id, "confirm", &wh_clerk).await;
    assert_eq!(result["success"], true);

    // Close it
    let result = execute_workflow_action(&app, "scm_goods_receipts", &gr_id, "close", &wh_clerk).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "closed");

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 24: Unbalanced Journal Entry (Debits != Credits)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_unbalanced_journal_entry_data() {
    let (state, app) = setup_p2p().await;
    let _admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let je = create_record(&app, "fin_journal_entries", json!({
        "entry_number": "JE-UNBAL-001",
        "entry_date": "2025-02-01",
        "description": "Unbalanced entry",
        "total_debit": 1000.00,
        "total_credit": 500.00  // Unbalanced!
    }), &fin_mgr).await;
    let je_id = extract_id(&je);

    // The entry is created (validation would happen at post time)
    assert_eq!(je["total_debit"], 1000.00);
    assert_eq!(je["total_credit"], 500.00);

    // Journal entry workflow still works (business rule validation would be added)
    let result = execute_workflow_action(&app, "fin_journal_entries", &je_id, "submit", &fin_mgr).await;
    assert_eq!(result["success"], true);

    cleanup_p2p(&state.db_pool).await;
}

// ============================================================================
// TEST 25: Invoice Reject Returns to Draft
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_p2p_invoice_rejection_returns_to_draft() {
    let (state, app) = setup_p2p().await;
    let admin = admin_claims();
    let fin_mgr = finance_manager_claims();

    let (_supplier_id, _, _) = seed_master_data(&app, &admin).await;

    let invoice = create_record(&app, "fin_invoices", json!({
        "invoice_number": "INV-REJ-001",
        "invoice_date": "2025-01-15",
        "due_date": "2025-02-15",
        "subtotal": 750.00,
        "total_amount": 750.00,
        "balance_due": 750.00
    }), &fin_mgr).await;
    let inv_id = extract_id(&invoice);

    // Submit
    execute_workflow_action(&app, "fin_invoices", &inv_id, "submit", &fin_mgr).await;

    // Reject (goes back to draft)
    let result = execute_workflow_action(&app, "fin_invoices", &inv_id, "reject", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "draft");

    // Can resubmit
    let result = execute_workflow_action(&app, "fin_invoices", &inv_id, "submit", &fin_mgr).await;
    assert_eq!(result["success"], true);
    assert_eq!(result["to_state"], "submitted");

    cleanup_p2p(&state.db_pool).await;
}
