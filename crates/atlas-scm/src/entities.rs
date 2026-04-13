//! SCM Entity Definitions

use atlas_core::schema::SchemaBuilder;
use atlas_core::schema::WorkflowBuilder;
use atlas_shared::EntityDefinition;

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
