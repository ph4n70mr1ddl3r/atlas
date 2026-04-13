//! SCM Entity Definitions

use atlas_core::schema::SchemaBuilder;
use atlas_core::schema::WorkflowBuilder;
use atlas_shared::EntityDefinition;

/// Supplier entity
pub fn supplier_definition() -> EntityDefinition {
    SchemaBuilder::new("suppliers", "Supplier")
        .plural_label("Suppliers")
        .table_name("scm_suppliers")
        .description("External suppliers and vendors")
        .icon("truck")
        .required_string("supplier_number", "Supplier Number")
        .required_string("name", "Supplier Name")
        .string("contact_name", "Contact Person")
        .email("email", "Email")
        .phone("phone", "Phone")
        .enumeration("category", "Category", vec![
            "raw_materials", "finished_goods", "services", "equipment", "software", "other"
        ])
        .string("tax_id", "Tax ID")
        .enumeration("payment_terms", "Default Payment Terms", vec![
            "net_15", "net_30", "net_45", "net_60", "due_on_receipt"
        ])
        .currency("credit_limit", "Credit Limit", "USD")
        .enumeration("rating", "Rating", vec!["a", "b", "c", "d", "f"])
        .address("address", "Address")
        .boolean("is_active", "Active")
        .build()
}

/// Purchase Order entity with approval workflow
pub fn purchase_order_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("po_approval_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("submitted", "Pending Approval")
        .working_state("approved", "Approved")
        .final_state("rejected", "Rejected")
        .working_state("ordered", "Ordered")
        .working_state("received", "Received")
        .final_state("closed", "Closed")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "submitted", "submit")
        .transition("draft", "cancelled", "cancel")
        .transition("submitted", "approved", "approve")
        .transition("submitted", "rejected", "reject")
        .transition("approved", "ordered", "order")
        .transition("approved", "cancelled", "cancel")
        .transition("ordered", "received", "receive")
        .transition("received", "closed", "close")
        .build();

    SchemaBuilder::new("purchase_orders", "Purchase Order")
        .plural_label("Purchase Orders")
        .table_name("scm_purchase_orders")
        .description("Purchase orders for procuring goods and services")
        .icon("file-text")
        .required_string("po_number", "PO Number")
        .reference("supplier_id", "Supplier", "suppliers")
        .date("order_date", "Order Date")
        .date("expected_date", "Expected Delivery")
        .enumeration("currency_code", "Currency", vec!["USD", "EUR", "GBP", "JPY", "CNY"])
        .currency("subtotal", "Subtotal", "USD")
        .currency("tax_amount", "Tax", "USD")
        .currency("total_amount", "Total", "USD")
        .enumeration("payment_terms", "Payment Terms", vec![
            "net_15", "net_30", "net_45", "net_60", "due_on_receipt"
        ])
        .address("shipping_address", "Shipping Address")
        .rich_text("notes", "Notes")
        .reference("approved_by", "Approved By", "employees")
        .workflow(workflow)
        .build()
}

/// Product entity
pub fn product_definition() -> EntityDefinition {
    SchemaBuilder::new("products", "Product")
        .plural_label("Products")
        .table_name("scm_products")
        .description("Products and materials")
        .icon("package")
        .required_string("sku", "SKU")
        .required_string("name", "Product Name")
        .string("description", "Description")
        .enumeration("product_type", "Type", vec![
            "physical", "digital", "service", "raw_material"
        ])
        .enumeration("category", "Category", vec![
            "raw_materials", "finished_goods", "consumables", "services"
        ])
        .reference("supplier_id", "Default Supplier", "suppliers")
        .decimal("unit_price", "Unit Price", 18, 2)
        .decimal("cost_price", "Cost Price", 18, 2)
        .integer("reorder_level", "Reorder Level")
        .integer("reorder_quantity", "Reorder Quantity")
        .string("unit_of_measure", "Unit of Measure")
        .decimal("weight", "Weight (kg)", 10, 3)
        .boolean("is_active", "Active")
        .build()
}

/// Inventory Item entity
pub fn inventory_item_definition() -> EntityDefinition {
    SchemaBuilder::new("inventory_items", "Inventory Item")
        .plural_label("Inventory Items")
        .table_name("scm_inventory")
        .description("Inventory tracking per warehouse")
        .icon("archive")
        .reference("product_id", "Product", "products")
        .reference("warehouse_id", "Warehouse", "warehouses")
        .integer("quantity_on_hand", "Quantity On Hand")
        .integer("quantity_reserved", "Quantity Reserved")
        .integer("quantity_available", "Quantity Available")
        .decimal("unit_cost", "Unit Cost", 18, 2)
        .string("location_code", "Location Code")
        .date("last_counted_date", "Last Counted")
        .build()
}

/// Warehouse entity
pub fn warehouse_definition() -> EntityDefinition {
    SchemaBuilder::new("warehouses", "Warehouse")
        .plural_label("Warehouses")
        .table_name("scm_warehouses")
        .description("Warehouse locations")
        .icon("warehouse")
        .required_string("name", "Warehouse Name")
        .required_string("code", "Code")
        .address("address", "Address")
        .reference("manager_id", "Manager", "employees")
        .boolean("is_active", "Active")
        .build()
}

/// Sales Order entity with workflow
pub fn sales_order_definition() -> EntityDefinition {
    let workflow = WorkflowBuilder::new("sales_order_workflow", "draft")
        .initial_state("draft", "Draft")
        .working_state("confirmed", "Confirmed")
        .working_state("processing", "Processing")
        .working_state("shipped", "Shipped")
        .final_state("delivered", "Delivered")
        .final_state("cancelled", "Cancelled")
        .transition("draft", "confirmed", "confirm")
        .transition("confirmed", "processing", "start_processing")
        .transition("confirmed", "cancelled", "cancel")
        .transition("processing", "shipped", "ship")
        .transition("shipped", "delivered", "deliver")
        .build();

    SchemaBuilder::new("sales_orders", "Sales Order")
        .plural_label("Sales Orders")
        .table_name("scm_sales_orders")
        .description("Customer sales orders")
        .icon("shopping-cart")
        .required_string("order_number", "Order Number")
        .reference("customer_id", "Customer", "customers")
        .date("order_date", "Order Date")
        .date("expected_delivery", "Expected Delivery")
        .currency("subtotal", "Subtotal", "USD")
        .currency("tax", "Tax", "USD")
        .currency("total", "Total", "USD")
        .enumeration("priority", "Priority", vec![
            "low", "normal", "high", "urgent"
        ])
        .reference("sales_rep_id", "Sales Rep", "employees")
        .workflow(workflow)
        .build()
}
