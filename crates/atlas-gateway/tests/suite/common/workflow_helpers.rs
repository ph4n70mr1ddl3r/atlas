//! Comprehensive E2E test helpers for Procure-to-Pay and Order-to-Cash workflows

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

use atlas_core::{
    SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine,
    SecurityEngine, AuditEngine,
    eventbus::NatsEventBus,
    schema::{SchemaBuilder, PostgresSchemaRepository},
    audit::PostgresAuditRepository,
};
use atlas_shared::{
    EntityDefinition, WorkflowDefinition,
    StateDefinition, StateType, TransitionDefinition,
};
use atlas_gateway::AppState;

pub use super::super::common::helpers::{
    TEST_JWT_SECRET, Claims, admin_claims, user_claims, auth_header,
};

// ============================================================================
// Role-specific claims
// ============================================================================

pub fn purchase_manager_claims() -> Claims {
    Claims {
        sub: "00000000-0000-0000-0000-000000000010".to_string(),
        email: "buyer@atlas.local".to_string(),
        name: "Purchase Manager".to_string(),
        roles: vec!["purchase_manager".to_string(), "user".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

pub fn finance_manager_claims() -> Claims {
    Claims {
        sub: "00000000-0000-0000-0000-000000000011".to_string(),
        email: "finance@atlas.local".to_string(),
        name: "Finance Manager".to_string(),
        roles: vec!["finance_manager".to_string(), "user".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

pub fn sales_manager_claims() -> Claims {
    Claims {
        sub: "00000000-0000-0000-0000-000000000012".to_string(),
        email: "sales@atlas.local".to_string(),
        name: "Sales Manager".to_string(),
        roles: vec!["sales_manager".to_string(), "user".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

pub fn warehouse_claims() -> Claims {
    Claims {
        sub: "00000000-0000-0000-0000-000000000013".to_string(),
        email: "warehouse@atlas.local".to_string(),
        name: "Warehouse Clerk".to_string(),
        roles: vec!["warehouse_clerk".to_string(), "user".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

pub fn sales_rep_claims() -> Claims {
    Claims {
        sub: "00000000-0000-0000-0000-000000000014".to_string(),
        email: "salesrep@atlas.local".to_string(),
        name: "Sales Rep".to_string(),
        roles: vec!["sales_rep".to_string(), "user".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

// ============================================================================
// Entity Definitions
// ============================================================================

pub fn supplier_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_suppliers", "Supplier")
        .plural_label("Suppliers")
        .table_name("scm_suppliers")
        .required_string("supplier_number", "Supplier Number")
        .required_string("name", "Name")
        .string("contact_name", "Contact Name")
        .email("email", "Email")
        .phone("phone", "Phone")
        .string("category", "Category")
        .string("tax_id", "Tax ID")
        .string("payment_terms", "Payment Terms")
        .decimal("credit_limit", "Credit Limit", 18, 2)
        .string("rating", "Rating")
        .boolean_default("is_active", "Active", true)
        .build()
}

pub fn product_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_products", "Product")
        .plural_label("Products")
        .table_name("scm_products")
        .required_string("sku", "SKU")
        .required_string("name", "Name")
        .string("description", "Description")
        .string("product_type", "Product Type")
        .string("category", "Category")
        .reference("supplier_id", "Supplier", "scm_suppliers")
        .decimal("unit_price", "Unit Price", 18, 2)
        .decimal("cost_price", "Cost Price", 18, 2)
        .integer("reorder_level", "Reorder Level")
        .string("unit_of_measure", "Unit of Measure")
        .boolean_default("is_active", "Active", true)
        .build()
}

pub fn warehouse_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_warehouses", "Warehouse")
        .plural_label("Warehouses")
        .table_name("scm_warehouses")
        .required_string("name", "Name")
        .required_string("code", "Code")
        .boolean_default("is_active", "Active", true)
        .build()
}

pub fn inventory_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_inventory", "Inventory")
        .plural_label("Inventory")
        .table_name("scm_inventory")
        .reference("product_id", "Product", "scm_products")
        .reference("warehouse_id", "Warehouse", "scm_warehouses")
        .integer("quantity_on_hand", "Qty on Hand")
        .integer("quantity_reserved", "Qty Reserved")
        .integer("quantity_available", "Qty Available")
        .decimal("unit_cost", "Unit Cost", 18, 2)
        .build()
}

pub fn purchase_order_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "purchase_order_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "submitted".into(), label: "Submitted".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "approved".into(), label: "Approved".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "received".into(), label: "Received".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "invoiced".into(), label: "Invoiced".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "closed".into(), label: "Closed".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "rejected".into(), label: "Rejected".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "cancelled".into(), label: "Cancelled".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "submit".into(), from_state: "draft".into(), to_state: "submitted".into(), action: "submit".into(), action_label: Some("Submit for Approval".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "approve".into(), from_state: "submitted".into(), to_state: "approved".into(), action: "approve".into(), action_label: Some("Approve".into()), guards: vec![], required_roles: vec!["purchase_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "reject".into(), from_state: "submitted".into(), to_state: "rejected".into(), action: "reject".into(), action_label: Some("Reject".into()), guards: vec![], required_roles: vec!["purchase_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "receive".into(), from_state: "approved".into(), to_state: "received".into(), action: "receive".into(), action_label: Some("Mark Received".into()), guards: vec![], required_roles: vec!["warehouse_clerk".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "invoice".into(), from_state: "received".into(), to_state: "invoiced".into(), action: "invoice".into(), action_label: Some("Create Invoice".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "close".into(), from_state: "invoiced".into(), to_state: "closed".into(), action: "close".into(), action_label: Some("Close".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "cancel_draft".into(), from_state: "draft".into(), to_state: "cancelled".into(), action: "cancel".into(), action_label: Some("Cancel".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "resubmit".into(), from_state: "rejected".into(), to_state: "draft".into(), action: "resubmit".into(), action_label: Some("Resubmit".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("scm_purchase_orders", "Purchase Order")
        .plural_label("Purchase Orders")
        .table_name("scm_purchase_orders")
        .required_string("po_number", "PO Number")
        .reference("supplier_id", "Supplier", "scm_suppliers")
        .date("order_date", "Order Date")
        .date("expected_date", "Expected Date")
        .string("currency_code", "Currency")
        .decimal("subtotal", "Subtotal", 18, 2)
        .decimal("tax_amount", "Tax", 18, 2)
        .decimal("total_amount", "Total", 18, 2)
        .string("payment_terms", "Payment Terms")
        .string("notes", "Notes")
        .workflow(workflow)
        .build()
}

pub fn purchase_order_line_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_purchase_order_lines", "Purchase Order Line")
        .plural_label("Purchase Order Lines")
        .table_name("scm_purchase_order_lines")
        .reference("purchase_order_id", "Purchase Order", "scm_purchase_orders")
        .integer("line_number", "Line Number")
        .reference("product_id", "Product", "scm_products")
        .string("description", "Description")
        .decimal("quantity", "Quantity", 12, 2)
        .decimal("unit_price", "Unit Price", 18, 2)
        .string("unit_of_measure", "UoM")
        .decimal("tax_rate", "Tax Rate", 5, 2)
        .decimal("tax_amount", "Tax Amount", 18, 2)
        .decimal("line_total", "Line Total", 18, 2)
        .decimal("received_quantity", "Received Qty", 12, 2)
        .build()
}

pub fn goods_receipt_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "goods_receipt_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "confirmed".into(), label: "Confirmed".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "closed".into(), label: "Closed".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "confirm".into(), from_state: "draft".into(), to_state: "confirmed".into(), action: "confirm".into(), action_label: Some("Confirm Receipt".into()), guards: vec![], required_roles: vec!["warehouse_clerk".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "close".into(), from_state: "confirmed".into(), to_state: "closed".into(), action: "close".into(), action_label: Some("Close".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("scm_goods_receipts", "Goods Receipt")
        .plural_label("Goods Receipts")
        .table_name("scm_goods_receipts")
        .required_string("receipt_number", "Receipt Number")
        .reference("purchase_order_id", "Purchase Order", "scm_purchase_orders")
        .reference("supplier_id", "Supplier", "scm_suppliers")
        .reference("warehouse_id", "Warehouse", "scm_warehouses")
        .date("receipt_date", "Receipt Date")
        .decimal("total_quantity", "Total Quantity", 12, 2)
        .string("notes", "Notes")
        .workflow(workflow)
        .build()
}

pub fn goods_receipt_line_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_goods_receipt_lines", "Goods Receipt Line")
        .plural_label("Goods Receipt Lines")
        .table_name("scm_goods_receipt_lines")
        .reference("goods_receipt_id", "Goods Receipt", "scm_goods_receipts")
        .integer("line_number", "Line Number")
        .reference("purchase_order_line_id", "PO Line", "scm_purchase_order_lines")
        .reference("product_id", "Product", "scm_products")
        .reference("warehouse_id", "Warehouse", "scm_warehouses")
        .decimal("quantity_received", "Received Qty", 12, 2)
        .decimal("quantity_accepted", "Accepted Qty", 12, 2)
        .decimal("quantity_rejected", "Rejected Qty", 12, 2)
        .build()
}

pub fn invoice_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "invoice_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "submitted".into(), label: "Submitted".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "approved".into(), label: "Approved".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "paid".into(), label: "Paid".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "void".into(), label: "Void".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "submit".into(), from_state: "draft".into(), to_state: "submitted".into(), action: "submit".into(), action_label: Some("Submit".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "approve".into(), from_state: "submitted".into(), to_state: "approved".into(), action: "approve".into(), action_label: Some("Approve".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "reject".into(), from_state: "submitted".into(), to_state: "draft".into(), action: "reject".into(), action_label: Some("Reject".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "mark_paid".into(), from_state: "approved".into(), to_state: "paid".into(), action: "mark_paid".into(), action_label: Some("Mark Paid".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "void".into(), from_state: "draft".into(), to_state: "void".into(), action: "void".into(), action_label: Some("Void".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("fin_invoices", "Invoice")
        .plural_label("Invoices")
        .table_name("fin_invoices")
        .required_string("invoice_number", "Invoice Number")
        .reference("customer_id", "Customer", "crm_customers")
        .date("invoice_date", "Invoice Date")
        .date("due_date", "Due Date")
        .string("status", "Status")
        .decimal("subtotal", "Subtotal", 18, 2)
        .decimal("tax_amount", "Tax", 18, 2)
        .decimal("total_amount", "Total", 18, 2)
        .decimal("amount_paid", "Paid", 18, 2)
        .decimal("balance_due", "Balance Due", 18, 2)
        .string("payment_terms", "Payment Terms")
        .string("notes", "Notes")
        .workflow(workflow)
        .build()
}

pub fn invoice_line_entity() -> EntityDefinition {
    SchemaBuilder::new("fin_invoice_lines", "Invoice Line")
        .plural_label("Invoice Lines")
        .table_name("fin_invoice_lines")
        .reference("invoice_id", "Invoice", "fin_invoices")
        .integer("line_number", "Line Number")
        .reference("product_id", "Product", "scm_products")
        .string("description", "Description")
        .decimal("quantity", "Quantity", 12, 2)
        .decimal("unit_price", "Unit Price", 18, 2)
        .decimal("tax_rate", "Tax Rate", 5, 2)
        .decimal("tax_amount", "Tax Amount", 18, 2)
        .decimal("line_total", "Line Total", 18, 2)
        .string("reference_type", "Reference Type")
        .build()
}

pub fn payment_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "payment_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "confirmed".into(), label: "Confirmed".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "reconciled".into(), label: "Reconciled".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "confirm".into(), from_state: "draft".into(), to_state: "confirmed".into(), action: "confirm".into(), action_label: Some("Confirm".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "reconcile".into(), from_state: "confirmed".into(), to_state: "reconciled".into(), action: "reconcile".into(), action_label: Some("Reconcile".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("fin_payments", "Payment")
        .plural_label("Payments")
        .table_name("fin_payments")
        .required_string("payment_number", "Payment Number")
        .string("payment_type", "Payment Type")
        .string("payment_method", "Payment Method")
        .reference("payer_id", "Payer", "crm_customers")
        .reference("payee_id", "Payee", "scm_suppliers")
        .reference("invoice_id", "Invoice", "fin_invoices")
        .decimal("amount", "Amount", 18, 2)
        .string("currency_code", "Currency")
        .date("payment_date", "Payment Date")
        .string("reference_number", "Reference")
        .string("bank_account", "Bank Account")
        .boolean_default("reconciled", "Reconciled", false)
        .string("notes", "Notes")
        .workflow(workflow)
        .build()
}

pub fn journal_entry_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "journal_entry_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "submitted".into(), label: "Submitted".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "posted".into(), label: "Posted".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "submit".into(), from_state: "draft".into(), to_state: "submitted".into(), action: "submit".into(), action_label: Some("Submit".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "post".into(), from_state: "submitted".into(), to_state: "posted".into(), action: "post".into(), action_label: Some("Post".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "reject".into(), from_state: "submitted".into(), to_state: "draft".into(), action: "reject".into(), action_label: Some("Reject".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("fin_journal_entries", "Journal Entry")
        .plural_label("Journal Entries")
        .table_name("fin_journal_entries")
        .required_string("entry_number", "Entry Number")
        .date("entry_date", "Entry Date")
        .string("description", "Description")
        .decimal("total_debit", "Total Debit", 18, 2)
        .decimal("total_credit", "Total Credit", 18, 2)
        .workflow(workflow)
        .build()
}

pub fn journal_entry_line_entity() -> EntityDefinition {
    SchemaBuilder::new("fin_journal_entry_lines", "Journal Entry Line")
        .plural_label("Journal Entry Lines")
        .table_name("fin_journal_entry_lines")
        .reference("journal_entry_id", "Journal Entry", "fin_journal_entries")
        .integer("line_number", "Line Number")
        .reference("account_id", "Account", "fin_chart_of_accounts")
        .string("description", "Description")
        .decimal("debit_amount", "Debit", 18, 2)
        .decimal("credit_amount", "Credit", 18, 2)
        .build()
}

pub fn chart_of_accounts_entity() -> EntityDefinition {
    SchemaBuilder::new("fin_chart_of_accounts", "Account")
        .plural_label("Chart of Accounts")
        .table_name("fin_chart_of_accounts")
        .required_string("account_number", "Account Number")
        .required_string("name", "Name")
        .required_string("account_type", "Type")
        .string("subtype", "Subtype")
        .boolean_default("is_active", "Active", true)
        .build()
}

// O2C entities

pub fn customer_entity() -> EntityDefinition {
    SchemaBuilder::new("crm_customers", "Customer")
        .plural_label("Customers")
        .table_name("crm_customers")
        .required_string("customer_number", "Customer Number")
        .required_string("name", "Name")
        .string("type", "Type")
        .string("industry", "Industry")
        .email("email", "Email")
        .phone("phone", "Phone")
        .string("status", "Status")
        .decimal("revenue", "Revenue", 18, 2)
        .integer("employee_count", "Employees")
        .build()
}

pub fn lead_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "lead_workflow".to_string(),
        initial_state: "new".to_string(),
        states: vec![
            StateDefinition { name: "new".into(), label: "New".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "contacted".into(), label: "Contacted".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "qualified".into(), label: "Qualified".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "converted".into(), label: "Converted".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "disqualified".into(), label: "Disqualified".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "contact".into(), from_state: "new".into(), to_state: "contacted".into(), action: "contact".into(), action_label: Some("Contact".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "qualify".into(), from_state: "contacted".into(), to_state: "qualified".into(), action: "qualify".into(), action_label: Some("Qualify".into()), guards: vec![], required_roles: vec!["sales_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "convert".into(), from_state: "qualified".into(), to_state: "converted".into(), action: "convert".into(), action_label: Some("Convert to Customer".into()), guards: vec![], required_roles: vec!["sales_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "disqualify".into(), from_state: "new".into(), to_state: "disqualified".into(), action: "disqualify".into(), action_label: Some("Disqualify".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "disqualify2".into(), from_state: "contacted".into(), to_state: "disqualified".into(), action: "disqualify".into(), action_label: Some("Disqualify".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("crm_leads", "Lead")
        .plural_label("Leads")
        .table_name("crm_leads")
        .required_string("first_name", "First Name")
        .required_string("last_name", "Last Name")
        .email("email", "Email")
        .phone("phone", "Phone")
        .string("company", "Company")
        .string("title", "Title")
        .string("source", "Source")
        .string("industry", "Industry")
        .decimal("estimated_value", "Est. Value", 18, 2)
        .string("notes", "Notes")
        .workflow(workflow)
        .build()
}

pub fn contact_entity() -> EntityDefinition {
    SchemaBuilder::new("crm_contacts", "Contact")
        .plural_label("Contacts")
        .table_name("crm_contacts")
        .required_string("first_name", "First Name")
        .required_string("last_name", "Last Name")
        .email("email", "Email")
        .phone("phone", "Phone")
        .string("title", "Title")
        .reference("customer_id", "Customer", "crm_customers")
        .boolean_default("is_primary", "Primary", false)
        .boolean_default("is_active", "Active", true)
        .build()
}

pub fn sales_order_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "sales_order_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "confirmed".into(), label: "Confirmed".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "processing".into(), label: "Processing".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "shipped".into(), label: "Shipped".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "invoiced".into(), label: "Invoiced".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "completed".into(), label: "Completed".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "cancelled".into(), label: "Cancelled".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "confirm".into(), from_state: "draft".into(), to_state: "confirmed".into(), action: "confirm".into(), action_label: Some("Confirm".into()), guards: vec![], required_roles: vec!["sales_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "process".into(), from_state: "confirmed".into(), to_state: "processing".into(), action: "process".into(), action_label: Some("Process".into()), guards: vec![], required_roles: vec!["warehouse_clerk".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "ship".into(), from_state: "processing".into(), to_state: "shipped".into(), action: "ship".into(), action_label: Some("Ship".into()), guards: vec![], required_roles: vec!["warehouse_clerk".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "invoice".into(), from_state: "shipped".into(), to_state: "invoiced".into(), action: "invoice".into(), action_label: Some("Create Invoice".into()), guards: vec![], required_roles: vec!["finance_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "complete".into(), from_state: "invoiced".into(), to_state: "completed".into(), action: "complete".into(), action_label: Some("Complete".into()), guards: vec![], required_roles: vec!["sales_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "cancel".into(), from_state: "draft".into(), to_state: "cancelled".into(), action: "cancel".into(), action_label: Some("Cancel".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "cancel_confirmed".into(), from_state: "confirmed".into(), to_state: "cancelled".into(), action: "cancel".into(), action_label: Some("Cancel".into()), guards: vec![], required_roles: vec!["sales_manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("scm_sales_orders", "Sales Order")
        .plural_label("Sales Orders")
        .table_name("scm_sales_orders")
        .required_string("order_number", "Order Number")
        .reference("customer_id", "Customer", "crm_customers")
        .date("order_date", "Order Date")
        .date("expected_delivery", "Expected Delivery")
        .decimal("subtotal", "Subtotal", 18, 2)
        .decimal("tax", "Tax", 18, 2)
        .decimal("total", "Total", 18, 2)
        .string("priority", "Priority")
        .workflow(workflow)
        .build()
}

pub fn sales_order_line_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_sales_order_lines", "Sales Order Line")
        .plural_label("Sales Order Lines")
        .table_name("scm_sales_order_lines")
        .reference("sales_order_id", "Sales Order", "scm_sales_orders")
        .integer("line_number", "Line Number")
        .reference("product_id", "Product", "scm_products")
        .string("description", "Description")
        .decimal("quantity", "Quantity", 12, 2)
        .decimal("unit_price", "Unit Price", 18, 2)
        .decimal("discount_percent", "Discount %", 5, 2)
        .decimal("tax_rate", "Tax Rate", 5, 2)
        .decimal("line_total", "Line Total", 18, 2)
        .decimal("shipped_quantity", "Shipped Qty", 12, 2)
        .build()
}

pub fn shipment_entity() -> EntityDefinition {
    let workflow = WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "shipment_workflow".to_string(),
        initial_state: "pending".to_string(),
        states: vec![
            StateDefinition { name: "pending".into(), label: "Pending".into(), state_type: StateType::Initial, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "shipped".into(), label: "Shipped".into(), state_type: StateType::Working, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "delivered".into(), label: "Delivered".into(), state_type: StateType::Final, entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "ship".into(), from_state: "pending".into(), to_state: "shipped".into(), action: "ship".into(), action_label: Some("Ship".into()), guards: vec![], required_roles: vec!["warehouse_clerk".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "deliver".into(), from_state: "shipped".into(), to_state: "delivered".into(), action: "deliver".into(), action_label: Some("Deliver".into()), guards: vec![], required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    };

    SchemaBuilder::new("scm_shipments", "Shipment")
        .plural_label("Shipments")
        .table_name("scm_shipments")
        .required_string("shipment_number", "Shipment Number")
        .reference("sales_order_id", "Sales Order", "scm_sales_orders")
        .reference("customer_id", "Customer", "crm_customers")
        .reference("warehouse_id", "Warehouse", "scm_warehouses")
        .date("shipment_date", "Ship Date")
        .date("estimated_delivery", "Est. Delivery")
        .date("actual_delivery", "Actual Delivery")
        .string("carrier", "Carrier")
        .string("tracking_number", "Tracking #")
        .string("notes", "Notes")
        .workflow(workflow)
        .build()
}

pub fn shipment_line_entity() -> EntityDefinition {
    SchemaBuilder::new("scm_shipment_lines", "Shipment Line")
        .plural_label("Shipment Lines")
        .table_name("scm_shipment_lines")
        .reference("shipment_id", "Shipment", "scm_shipments")
        .integer("line_number", "Line Number")
        .reference("sales_order_line_id", "SO Line", "scm_sales_order_lines")
        .reference("product_id", "Product", "scm_products")
        .decimal("quantity_shipped", "Shipped Qty", 12, 2)
        .build()
}

// ============================================================================
// Test State Setup
// ============================================================================

pub fn all_p2p_entities() -> Vec<EntityDefinition> {
    vec![
        supplier_entity(),
        product_entity(),
        warehouse_entity(),
        inventory_entity(),
        purchase_order_entity(),
        purchase_order_line_entity(),
        goods_receipt_entity(),
        goods_receipt_line_entity(),
        invoice_entity(),
        invoice_line_entity(),
        payment_entity(),
        journal_entry_entity(),
        journal_entry_line_entity(),
        chart_of_accounts_entity(),
    ]
}

pub fn all_o2c_entities() -> Vec<EntityDefinition> {
    vec![
        customer_entity(),
        lead_entity(),
        contact_entity(),
        product_entity(),
        warehouse_entity(),
        inventory_entity(),
        sales_order_entity(),
        sales_order_line_entity(),
        shipment_entity(),
        shipment_line_entity(),
        invoice_entity(),
        invoice_line_entity(),
        payment_entity(),
        journal_entry_entity(),
        journal_entry_line_entity(),
        chart_of_accounts_entity(),
    ]
}

pub async fn build_workflow_test_state() -> Arc<AppState> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://atlas:atlas@localhost:5432/atlas".to_string());

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let schema_engine = Arc::new(SchemaEngine::new(Arc::new(PostgresSchemaRepository::new(db_pool.clone()))));
    let audit_engine = Arc::new(AuditEngine::new(Arc::new(PostgresAuditRepository::new(db_pool.clone()))));
    let workflow_engine = Arc::new(WorkflowEngine::new());
    let validation_engine = Arc::new(ValidationEngine::new());
    let formula_engine = Arc::new(FormulaEngine::new());
    let security_engine = Arc::new(SecurityEngine::new());
    let event_bus = Arc::new(NatsEventBus::noop("atlas-gateway-test"));

    let notification_engine = Arc::new(atlas_core::NotificationEngine::new(Arc::new(
        atlas_core::notification::PostgresNotificationRepository::new(db_pool.clone()),
    )));
    let approval_engine = Arc::new(atlas_core::ApprovalEngine::new(Arc::new(
        atlas_core::approval::PostgresApprovalRepository::new(db_pool.clone()),
    )));
    let period_close_engine = Arc::new(atlas_core::PeriodCloseEngine::new(Arc::new(
        atlas_core::period_close::PostgresPeriodCloseRepository::new(db_pool.clone()),
    )));

    let currency_engine = Arc::new(atlas_core::CurrencyEngine::new(Arc::new(
        atlas_core::currency::PostgresCurrencyRepository::new(db_pool.clone()),
    )));

    let tax_engine = Arc::new(atlas_core::TaxEngine::new(Arc::new(
        atlas_core::tax::PostgresTaxRepository::new(db_pool.clone()),
    )));

    let intercompany_engine = Arc::new(atlas_core::IntercompanyEngine::new(Arc::new(
        atlas_core::intercompany::PostgresIntercompanyRepository::new(db_pool.clone()),
    )));

    let reconciliation_engine = Arc::new(atlas_core::ReconciliationEngine::new(Arc::new(
        atlas_core::reconciliation::PostgresReconciliationRepository::new(db_pool.clone()),
    )));

    let expense_engine = Arc::new(atlas_core::ExpenseEngine::new(Arc::new(
        atlas_core::expense::PostgresExpenseRepository::new(db_pool.clone()),
    )));

    let budget_engine = Arc::new(atlas_core::BudgetEngine::new(Arc::new(
        atlas_core::budget::PostgresBudgetRepository::new(db_pool.clone()),
    )));

    let fixed_asset_engine = Arc::new(atlas_core::FixedAssetEngine::new(Arc::new(
        atlas_core::fixed_assets::PostgresFixedAssetRepository::new(db_pool.clone()),
    )));

    let sla_engine = Arc::new(atlas_core::SubledgerAccountingEngine::new(Arc::new(
        atlas_core::subledger_accounting::PostgresSubledgerAccountingRepository::new(db_pool.clone()),
    )));

    let encumbrance_engine = Arc::new(atlas_core::EncumbranceEngine::new(Arc::new(
        atlas_core::encumbrance::PostgresEncumbranceRepository::new(db_pool.clone()),
    )));

    let cash_management_engine = Arc::new(atlas_core::CashManagementEngine::new(Arc::new(
        atlas_core::cash_management::PostgresCashManagementRepository::new(db_pool.clone()),
    )));

    let sourcing_engine = Arc::new(atlas_core::SourcingEngine::new(Arc::new(
        atlas_core::sourcing::PostgresSourcingRepository::new(db_pool.clone()),
    )));

    let lease_accounting_engine = Arc::new(atlas_core::LeaseAccountingEngine::new(Arc::new(
        atlas_core::lease::PostgresLeaseAccountingRepository::new(db_pool.clone()),
    )));

    let project_costing_engine = Arc::new(atlas_core::ProjectCostingEngine::new(Arc::new(
        atlas_core::project_costing::PostgresProjectCostingRepository::new(db_pool.clone()),
    )));

    let cost_allocation_engine = Arc::new(atlas_core::CostAllocationEngine::new(Arc::new(
        atlas_core::cost_allocation::PostgresCostAllocationRepository::new(db_pool.clone()),
    )));

    let financial_reporting_engine = Arc::new(atlas_core::FinancialReportingEngine::new(Arc::new(
        atlas_core::financial_reporting::PostgresFinancialReportingRepository::new(db_pool.clone()),
    )));

    let multi_book_engine = Arc::new(atlas_core::MultiBookAccountingEngine::new(Arc::new(
        atlas_core::multi_book::PostgresMultiBookAccountingRepository::new(db_pool.clone()),
    )));

    let procurement_contract_engine = Arc::new(atlas_core::ProcurementContractEngine::new(Arc::new(
        atlas_core::procurement_contracts::PostgresProcurementContractRepository::new(db_pool.clone()),
    )));

    let inventory_engine = Arc::new(atlas_core::InventoryEngine::new(Arc::new(
        atlas_core::inventory::PostgresInventoryRepository::new(db_pool.clone()),
    )));

    let customer_returns_engine = Arc::new(atlas_core::CustomerReturnsEngine::new(Arc::new(
        atlas_core::customer_returns::PostgresCustomerReturnsRepository::new(db_pool.clone()),
    )));

    let pricing_engine = Arc::new(atlas_core::PricingEngine::new(Arc::new(
        atlas_core::pricing::PostgresPricingRepository::new(db_pool.clone()),
    )));

    let sales_commission_engine = Arc::new(atlas_core::SalesCommissionEngine::new(Arc::new(
        atlas_core::sales_commission::PostgresSalesCommissionRepository::new(db_pool.clone()),
    )));

    let treasury_engine = Arc::new(atlas_core::TreasuryEngine::new(Arc::new(
        atlas_core::treasury::PostgresTreasuryRepository::new(db_pool.clone()),
    )));

    let grant_management_engine = Arc::new(atlas_core::GrantManagementEngine::new(Arc::new(
        atlas_core::grant_management::PostgresGrantManagementRepository::new(db_pool.clone()),
    )));

    let supplier_qualification_engine = Arc::new(atlas_core::SupplierQualificationEngine::new(Arc::new(
        atlas_core::supplier_qualification::PostgresSupplierQualificationRepository::new(db_pool.clone()),
    )));

    let recurring_journal_engine = Arc::new(atlas_core::RecurringJournalEngine::new(Arc::new(
        atlas_core::recurring_journal::PostgresRecurringJournalRepository::new(db_pool.clone()),
    )));

    let manual_journal_engine = Arc::new(atlas_core::ManualJournalEngine::new(Arc::new(
        atlas_core::manual_journal::PostgresManualJournalRepository::new(db_pool.clone()),
    )));

    let dff_engine = Arc::new(atlas_core::DescriptiveFlexfieldEngine::new(Arc::new(
        atlas_core::descriptive_flexfield::PostgresDescriptiveFlexfieldRepository::new(db_pool.clone()),
    )));

    let cvr_engine = Arc::new(atlas_core::CrossValidationEngine::new(Arc::new(
        atlas_core::cross_validation::PostgresCrossValidationRepository::new(db_pool.clone()),
    )));

    let scheduled_process_engine = Arc::new(atlas_core::ScheduledProcessEngine::new(Arc::new(
        atlas_core::scheduled_process::PostgresScheduledProcessRepository::new(db_pool.clone()),
    )));

    let sod_engine = Arc::new(atlas_core::SegregationOfDutiesEngine::new(Arc::new(
        atlas_core::segregation_of_duties::PostgresSegregationOfDutiesRepository::new(db_pool.clone()),
    )));

    let allocation_engine = Arc::new(atlas_core::AllocationEngine::new(Arc::new(
        atlas_core::allocation::PostgresAllocationRepository::new(db_pool.clone()),
    )));

    let state = AppState {
        db_pool: db_pool.clone(),
        schema_engine,
        workflow_engine,
        validation_engine,
        formula_engine,
        security_engine,
        audit_engine,
        notification_engine,
        approval_engine,
        period_close_engine,
        currency_engine,
        tax_engine,
        intercompany_engine,
    reconciliation_engine,
        expense_engine,
        budget_engine,
        fixed_asset_engine,
        sla_engine,
        encumbrance_engine,
        cash_management_engine,
        sourcing_engine,
        lease_accounting_engine,
        project_costing_engine,
        cost_allocation_engine,
        financial_reporting_engine,
        multi_book_engine,
        procurement_contract_engine,
        inventory_engine,
        customer_returns_engine,
        pricing_engine,
        sales_commission_engine,
        treasury_engine,
        grant_management_engine,
        supplier_qualification_engine,
        recurring_journal_engine,
        manual_journal_engine,
        dff_engine,
        cvr_engine,
        scheduled_process_engine,
        sod_engine,
        allocation_engine,
        currency_revaluation_engine: Arc::new(atlas_core::CurrencyRevaluationEngine::new(Arc::new(
            atlas_core::currency_revaluation::PostgresCurrencyRevaluationRepository::new(db_pool.clone()),
        ))),
        purchase_requisition_engine: Arc::new(atlas_core::PurchaseRequisitionEngine::new(Arc::new(
            atlas_core::purchase_requisition::PostgresPurchaseRequisitionRepository::new(db_pool.clone()),
        ))),
        corporate_card_engine: Arc::new(atlas_core::CorporateCardEngine::new(Arc::new(
            atlas_core::corporate_card::PostgresCorporateCardRepository::new(db_pool.clone()),
        ))),
        benefits_engine: Arc::new(atlas_core::BenefitsEngine::new(Arc::new(
            atlas_core::benefits::PostgresBenefitsRepository::new(db_pool.clone()),
        ))),
        performance_engine: Arc::new(atlas_core::PerformanceEngine::new(Arc::new(
            atlas_core::performance::PostgresPerformanceRepository::new(db_pool.clone()),
        ))),
        credit_management_engine: Arc::new(atlas_core::CreditManagementEngine::new(Arc::new(
            atlas_core::credit_management::PostgresCreditManagementRepository::new(db_pool.clone()),
        ))),
        product_information_engine: Arc::new(atlas_core::ProductInformationEngine::new(Arc::new(
            atlas_core::product_information::PostgresProductInformationRepository::new(db_pool.clone()),
        ))),
        transfer_pricing_engine: Arc::new(atlas_core::TransferPricingEngine::new(Arc::new(
            atlas_core::transfer_pricing::PostgresTransferPricingRepository::new(db_pool.clone()),
        ))),
        approval_delegation_engine: Arc::new(atlas_core::ApprovalDelegationEngine::new(Arc::new(
            atlas_core::approval_delegation::PostgresApprovalDelegationRepository::new(db_pool.clone()),
        ))),
        order_management_engine: Arc::new(atlas_core::OrderManagementEngine::new(Arc::new(
            atlas_core::order_management::PostgresOrderManagementRepository::new(db_pool.clone()),
        ))),
        manufacturing_engine: Arc::new(atlas_core::ManufacturingEngine::new(Arc::new(
            atlas_core::manufacturing::PostgresManufacturingRepository::new(db_pool.clone()),
        ))),
        warehouse_management_engine: Arc::new(atlas_core::WarehouseManagementEngine::new(Arc::new(
            atlas_core::warehouse_management::PostgresWarehouseManagementRepository::new(db_pool.clone()),
        ))),
        absence_engine: Arc::new(atlas_core::AbsenceEngine::new(Arc::new(
            atlas_core::absence::PostgresAbsenceRepository::new(db_pool.clone()),
        ))),
        event_bus,
        jwt_secret: TEST_JWT_SECRET.to_string(),
    };

    let state = Arc::new(state);
    atlas_gateway::state::APP_STATE.set(state.clone()).ok();
    state
}

pub async fn setup_p2p_entities(state: &Arc<AppState>) {
    for entity in all_p2p_entities() {
        state.schema_engine.upsert_entity(entity).await.unwrap();
    }
    for entity in all_p2p_entities() {
        if let Some(e) = state.schema_engine.get_entity(&entity.name) {
            if let Some(ref wf) = e.workflow {
                state.workflow_engine.load_workflow(wf.clone()).await.unwrap();
            }
        }
    }
}

pub async fn setup_o2c_entities(state: &Arc<AppState>) {
    for entity in all_o2c_entities() {
        state.schema_engine.upsert_entity(entity).await.unwrap();
    }
    for entity in all_o2c_entities() {
        if let Some(e) = state.schema_engine.get_entity(&entity.name) {
            if let Some(ref wf) = e.workflow {
                state.workflow_engine.load_workflow(wf.clone()).await.unwrap();
            }
        }
    }
}

pub fn build_app(state: Arc<AppState>) -> Router {
    use tower_http::cors::{CorsLayer, Any};
    let cors = CorsLayer::new().allow_methods(Any).allow_headers(Any).allow_origin(Any);
    Router::new()
        .nest("/api/v1", atlas_gateway::handlers::api_routes())
        .nest("/api/admin", atlas_gateway::handlers::admin_routes())
        .route("/health", axum::routing::get(atlas_gateway::handlers::health_check))
        .route("/api/v1/auth/login", axum::routing::post(atlas_gateway::handlers::login))
        .layer(cors)
        .with_state(state)
}

// ============================================================================
// API Helper Functions
// ============================================================================

pub async fn create_record(
    app: &Router, entity: &str, values: serde_json::Value, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder().method("POST")
            .uri(format!("/api/v1/{}", entity))
            .header("Content-Type", "application/json")
            .header(k, v)
            .body(Body::from(serde_json::to_string(&json!({
                "entity": entity, "values": values
            })).unwrap()))
            .unwrap()
    ).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&body);
        panic!("Create {} failed with {}: {}", entity, status, body_str);
    }
    serde_json::from_slice(&body).unwrap()
}

pub async fn get_record(
    app: &Router, entity: &str, id: &str, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/{}/{}", entity, id))
            .header(k, v)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn update_record(
    app: &Router, entity: &str, id: &str, values: serde_json::Value, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder().method("PUT")
            .uri(format!("/api/v1/{}/{}", entity, id))
            .header("Content-Type", "application/json")
            .header(k, v)
            .body(Body::from(serde_json::to_string(&json!({
                "entity": entity, "id": id, "values": values
            })).unwrap()))
            .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn delete_record(
    app: &Router, entity: &str, id: &str, claims: &Claims,
) -> StatusCode {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder().method("DELETE")
            .uri(format!("/api/v1/{}/{}", entity, id))
            .header(k, v)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    resp.status()
}

pub async fn execute_workflow_action(
    app: &Router, entity: &str, id: &str, action: &str, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder().method("POST")
            .uri(format!("/api/v1/{}/{}/{}", entity, id, action))
            .header("Content-Type", "application/json")
            .header(k, v)
            .body(Body::from(serde_json::to_string(&json!({
                "action": action, "comment": format!("E2E test: {}", action)
            })).unwrap()))
            .unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn execute_workflow_action_expect_status(
    app: &Router, entity: &str, id: &str, action: &str, claims: &Claims, expected: StatusCode,
) -> StatusCode {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder().method("POST")
            .uri(format!("/api/v1/{}/{}/{}", entity, id, action))
            .header("Content-Type", "application/json")
            .header(k, v)
            .body(Body::from(serde_json::to_string(&json!({
                "action": action, "comment": "test"
            })).unwrap()))
            .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), expected, "Workflow action '{}' expected {:?} but got {:?}", action, expected, resp.status());
    resp.status()
}

pub async fn get_transitions(
    app: &Router, entity: &str, id: &str, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/{}/{}/transitions", entity, id))
            .header(k, v)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn get_history(
    app: &Router, entity: &str, id: &str, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/{}/{}/history", entity, id))
            .header(k, v)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn list_records(
    app: &Router, entity: &str, claims: &Claims,
) -> serde_json::Value {
    let (k, v) = auth_header(claims);
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/{}", entity))
            .header(k, v)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub fn extract_id(record: &serde_json::Value) -> String {
    record.get("id")
        .and_then(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .or_else(|| Some(v.to_string()))
        })
        .unwrap_or_else(|| panic!("Record has no 'id' field: {:?}", record))
}

// ============================================================================
// Cleanup
// ============================================================================

pub async fn cleanup_p2p(pool: &sqlx::PgPool) {
    // Delete data in dependency order (child tables first) to respect FK constraints
    let tables = [
        "fin_journal_entry_lines", "fin_journal_entries",
        "fin_invoice_lines", "fin_invoices",
        "fin_payments",
        "scm_goods_receipt_lines", "scm_goods_receipts",
        "scm_purchase_order_lines", "scm_purchase_orders",
        "scm_inventory", "scm_products", "scm_suppliers", "scm_warehouses",
        "fin_chart_of_accounts",
    ];
    for t in &tables {
        sqlx::query(&format!("DELETE FROM {}", t)).execute(pool).await.ok();
    }
    // Clean workflow states and audit log for test entities
    sqlx::query("DELETE FROM _atlas.workflow_states").execute(pool).await.ok();
    // Clean entity defs from schema
    let entities = [
        "scm_suppliers", "scm_products", "scm_warehouses", "scm_inventory",
        "scm_purchase_orders", "scm_purchase_order_lines",
        "scm_goods_receipts", "scm_goods_receipt_lines",
        "fin_invoices", "fin_invoice_lines", "fin_payments",
        "fin_journal_entries", "fin_journal_entry_lines",
        "fin_chart_of_accounts",
    ];
    for e in &entities {
        sqlx::query("DELETE FROM _atlas.entities WHERE name = $1")
            .bind(e).execute(pool).await.ok();
        sqlx::query("DELETE FROM _atlas.audit_log WHERE entity_type = $1")
            .bind(e).execute(pool).await.ok();
    }
}

pub async fn cleanup_o2c(pool: &sqlx::PgPool) {
    // Delete data in dependency order (child tables first) to respect FK constraints
    let tables = [
        "fin_journal_entry_lines", "fin_journal_entries",
        "fin_invoice_lines", "fin_invoices",
        "fin_payments",
        "scm_shipment_lines", "scm_shipments",
        "scm_sales_order_lines", "scm_sales_orders",
        "scm_inventory", "scm_products", "scm_warehouses",
        "crm_contacts", "crm_leads", "crm_customers",
        "fin_chart_of_accounts",
    ];
    for t in &tables {
        sqlx::query(&format!("DELETE FROM {}", t)).execute(pool).await.ok();
    }
    // Clean workflow states for test entities
    sqlx::query("DELETE FROM _atlas.workflow_states").execute(pool).await.ok();
    let entities = [
        "crm_customers", "crm_leads", "crm_contacts",
        "scm_sales_orders", "scm_sales_order_lines",
        "scm_shipments", "scm_shipment_lines",
        "scm_products", "scm_warehouses", "scm_inventory",
        "fin_invoices", "fin_invoice_lines", "fin_payments",
        "fin_journal_entries", "fin_journal_entry_lines",
        "fin_chart_of_accounts",
    ];
    for e in &entities {
        sqlx::query("DELETE FROM _atlas.entities WHERE name = $1")
            .bind(e).execute(pool).await.ok();
        sqlx::query("DELETE FROM _atlas.audit_log WHERE entity_type = $1")
            .bind(e).execute(pool).await.ok();
    }
}
