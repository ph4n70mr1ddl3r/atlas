-- Atlas ERP - Seed Initial Declarative Configuration
-- This creates the base entities that form the foundation of the ERP

-- ============================================================================
-- Organization Entity
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000001',
    'organizations',
    'Organization',
    'Organizations',
    'org_entities',
    'Business organizations or legal entities',
    '[
        {"name": "name", "label": "Organization Name", "field_type": {"type": "string", "max_length": 200}, "is_required": true, "is_searchable": true, "display_order": 1},
        {"name": "code", "label": "Code", "field_type": {"type": "string", "max_length": 50}, "is_required": true, "is_unique": true, "display_order": 2},
        {"name": "type", "label": "Type", "field_type": {"type": "enum", "values": ["company", "division", "department", "branch"]}, "display_order": 3},
        {"name": "tax_id", "label": "Tax ID", "field_type": {"type": "string", "max_length": 50}, "display_order": 4},
        {"name": "phone", "label": "Phone", "field_type": {"type": "phone"}, "display_order": 5},
        {"name": "email", "label": "Email", "field_type": {"type": "email"}, "display_order": 6},
        {"name": "website", "label": "Website", "field_type": {"type": "url"}, "display_order": 7},
        {"name": "address", "label": "Address", "field_type": {"type": "address"}, "display_order": 8},
        {"name": "is_active", "label": "Active", "field_type": {"type": "boolean"}, "default_value": true, "display_order": 9}
    ]'::jsonb,
    true,
    true
);

-- ============================================================================
-- Employee Entity (HCM Foundation)
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, workflow, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000002',
    'employees',
    'Employee',
    'Employees',
    'hcm_employees',
    'Employee records',
    '[
        {"name": "employee_number", "label": "Employee Number", "field_type": {"type": "string", "max_length": 50}, "is_required": true, "is_unique": true, "display_order": 1},
        {"name": "first_name", "label": "First Name", "field_type": {"type": "string", "max_length": 100}, "is_required": true, "display_order": 2},
        {"name": "last_name", "label": "Last Name", "field_type": {"type": "string", "max_length": 100}, "is_required": true, "display_order": 3},
        {"name": "middle_name", "label": "Middle Name", "field_type": {"type": "string", "max_length": 100}, "display_order": 4},
        {"name": "email", "label": "Email", "field_type": {"type": "email"}, "is_required": true, "display_order": 5},
        {"name": "phone", "label": "Phone", "field_type": {"type": "phone"}, "display_order": 6},
        {"name": "date_of_birth", "label": "Date of Birth", "field_type": {"type": "date"}, "display_order": 7},
        {"name": "hire_date", "label": "Hire Date", "field_type": {"type": "date"}, "is_required": true, "display_order": 8},
        {"name": "termination_date", "label": "Termination Date", "field_type": {"type": "date"}, "display_order": 9},
        {"name": "department_id", "label": "Department", "field_type": {"type": "reference", "entity": "departments"}, "display_order": 10},
        {"name": "position_id", "label": "Position", "field_type": {"type": "reference", "entity": "positions"}, "display_order": 11},
        {"name": "manager_id", "label": "Manager", "field_type": {"type": "reference", "entity": "employees"}, "display_order": 12},
        {"name": "employment_type", "label": "Employment Type", "field_type": {"type": "enum", "values": ["full_time", "part_time", "contractor", "intern"]}, "default_value": "\"full_time\"", "display_order": 13},
        {"name": "status", "label": "Status", "field_type": {"type": "enum", "values": ["active", "inactive", "on_leave", "terminated"]}, "default_value": "\"active\"", "display_order": 14},
        {"name": "address", "label": "Address", "field_type": {"type": "address"}, "display_order": 15}
    ]'::jsonb,
    '{
        "name": "employee_lifecycle",
        "initial_state": "onboarding",
        "states": [
            {"name": "onboarding", "label": "Onboarding", "state_type": "working"},
            {"name": "active", "label": "Active", "state_type": "working"},
            {"name": "on_leave", "label": "On Leave", "state_type": "working"},
            {"name": "offboarding", "label": "Offboarding", "state_type": "working"},
            {"name": "terminated", "label": "Terminated", "state_type": "final"}
        ],
        "transitions": [
            {"from_state": "onboarding", "to_state": "active", "action": "activate", "action_label": "Complete Onboarding"},
            {"from_state": "active", "to_state": "on_leave", "action": "start_leave", "action_label": "Start Leave"},
            {"from_state": "on_leave", "to_state": "active", "action": "end_leave", "action_label": "End Leave"},
            {"from_state": "active", "to_state": "offboarding", "action": "start_offboarding", "action_label": "Start Offboarding"},
            {"from_state": "active", "to_state": "onboarding", "action": "rehire", "action_label": "Rehire"},
            {"from_state": "offboarding", "to_state": "terminated", "action": "terminate", "action_label": "Terminate"}
        ],
        "is_active": true
    }'::jsonb,
    true,
    true
);

-- ============================================================================
-- Department Entity
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000003',
    'departments',
    'Department',
    'Departments',
    'hcm_departments',
    'Organizational departments',
    '[
        {"name": "name", "label": "Department Name", "field_type": {"type": "string", "max_length": 100}, "is_required": true, "display_order": 1},
        {"name": "code", "label": "Code", "field_type": {"type": "string", "max_length": 20}, "is_required": true, "is_unique": true, "display_order": 2},
        {"name": "parent_id", "label": "Parent Department", "field_type": {"type": "reference", "entity": "departments"}, "display_order": 3},
        {"name": "manager_id", "label": "Manager", "field_type": {"type": "reference", "entity": "employees"}, "display_order": 4},
        {"name": "cost_center", "label": "Cost Center", "field_type": {"type": "string", "max_length": 50}, "display_order": 5},
        {"name": "is_active", "label": "Active", "field_type": {"type": "boolean"}, "default_value": true, "display_order": 6}
    ]'::jsonb,
    true,
    true
);

-- ============================================================================
-- Position Entity
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000004',
    'positions',
    'Position',
    'Positions',
    'hcm_positions',
    'Job positions within the organization',
    '[
        {"name": "title", "label": "Job Title", "field_type": {"type": "string", "max_length": 100}, "is_required": true, "display_order": 1},
        {"name": "code", "label": "Code", "field_type": {"type": "string", "max_length": 20}, "is_required": true, "is_unique": true, "display_order": 2},
        {"name": "department_id", "label": "Department", "field_type": {"type": "reference", "entity": "departments"}, "display_order": 3},
        {"name": "level", "label": "Job Level", "field_type": {"type": "enum", "values": ["entry", "mid", "senior", "lead", "manager", "director", "executive"]}, "display_order": 4},
        {"name": "min_salary", "label": "Minimum Salary", "field_type": {"type": "currency", "code": "USD"}, "display_order": 5},
        {"name": "max_salary", "label": "Maximum Salary", "field_type": {"type": "currency", "code": "USD"}, "display_order": 6},
        {"name": "is_active", "label": "Active", "field_type": {"type": "boolean"}, "default_value": true, "display_order": 7}
    ]'::jsonb,
    true,
    true
);

-- ============================================================================
-- Purchase Order Entity (Financials + SCM)
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, workflow, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000005',
    'purchase_orders',
    'Purchase Order',
    'Purchase Orders',
    'fin_purchase_orders',
    'Purchase orders for procuring goods and services',
    '[
        {"name": "po_number", "label": "PO Number", "field_type": {"type": "string", "max_length": 50}, "is_required": true, "is_unique": true, "display_order": 1},
        {"name": "supplier_id", "label": "Supplier", "field_type": {"type": "reference", "entity": "suppliers"}, "is_required": true, "display_order": 2},
        {"name": "order_date", "label": "Order Date", "field_type": {"type": "date"}, "is_required": true, "display_order": 3},
        {"name": "expected_date", "label": "Expected Delivery Date", "field_type": {"type": "date"}, "display_order": 4},
        {"name": "status", "label": "Status", "field_type": {"type": "enum", "values": ["draft", "submitted", "approved", "rejected", "ordered", "received", "closed", "cancelled"]}, "default_value": "\"draft\"", "display_order": 5},
        {"name": "currency_code", "label": "Currency", "field_type": {"type": "enum", "values": ["USD", "EUR", "GBP", "JPY", "CNY"]}, "default_value": "\"USD\"", "display_order": 6},
        {"name": "subtotal", "label": "Subtotal", "field_type": {"type": "currency", "code": "USD"}, "is_read_only": true, "display_order": 7},
        {"name": "tax_amount", "label": "Tax Amount", "field_type": {"type": "currency", "code": "USD"}, "is_read_only": true, "display_order": 8},
        {"name": "total_amount", "label": "Total Amount", "field_type": {"type": "currency", "code": "USD"}, "is_read_only": true, "display_order": 9},
        {"name": "payment_terms", "label": "Payment Terms", "field_type": {"type": "enum", "values": ["net_15", "net_30", "net_45", "net_60", "due_on_receipt"]}, "display_order": 10},
        {"name": "shipping_address", "label": "Shipping Address", "field_type": {"type": "address"}, "display_order": 11},
        {"name": "notes", "label": "Notes", "field_type": {"type": "rich_text"}, "display_order": 12},
        {"name": "approved_by", "label": "Approved By", "field_type": {"type": "reference", "entity": "employees"}, "is_read_only": true, "display_order": 13},
        {"name": "approved_at", "label": "Approved At", "field_type": {"type": "date_time"}, "is_read_only": true, "display_order": 14}
    ]'::jsonb,
    '{
        "name": "po_approval_workflow",
        "initial_state": "draft",
        "states": [
            {"name": "draft", "label": "Draft", "state_type": "initial"},
            {"name": "submitted", "label": "Pending Approval", "state_type": "working"},
            {"name": "approved", "label": "Approved", "state_type": "working"},
            {"name": "rejected", "label": "Rejected", "state_type": "final"},
            {"name": "ordered", "label": "Ordered", "state_type": "working"},
            {"name": "received", "label": "Received", "state_type": "working"},
            {"name": "closed", "label": "Closed", "state_type": "final"},
            {"name": "cancelled", "label": "Cancelled", "state_type": "final"}
        ],
        "transitions": [
            {"from_state": "draft", "to_state": "submitted", "action": "submit", "action_label": "Submit for Approval"},
            {"from_state": "draft", "to_state": "cancelled", "action": "cancel", "action_label": "Cancel"},
            {"from_state": "submitted", "to_state": "approved", "action": "approve", "action_label": "Approve", "required_roles": ["purchase_manager", "finance_manager", "admin"]},
            {"from_state": "submitted", "to_state": "rejected", "action": "reject", "action_label": "Reject", "required_roles": ["purchase_manager", "finance_manager", "admin"]},
            {"from_state": "approved", "to_state": "ordered", "action": "order", "action_label": "Send to Supplier"},
            {"from_state": "approved", "to_state": "cancelled", "action": "cancel", "action_label": "Cancel"},
            {"from_state": "ordered", "to_state": "received", "action": "receive", "action_label": "Mark as Received"},
            {"from_state": "received", "to_state": "closed", "action": "close", "action_label": "Close PO"},
            {"from_state": "received", "to_state": "ordered", "action": "reopen", "action_label": "Reopen for Partial"}
        ],
        "is_active": true
    }'::jsonb,
    true,
    true
);

-- ============================================================================
-- Supplier Entity
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000006',
    'suppliers',
    'Supplier',
    'Suppliers',
    'scm_suppliers',
    'External suppliers and vendors',
    '[
        {"name": "supplier_number", "label": "Supplier Number", "field_type": {"type": "string", "max_length": 50}, "is_required": true, "is_unique": true, "display_order": 1},
        {"name": "name", "label": "Supplier Name", "field_type": {"type": "string", "max_length": 200}, "is_required": true, "is_searchable": true, "display_order": 2},
        {"name": "contact_name", "label": "Contact Person", "field_type": {"type": "string", "max_length": 100}, "display_order": 3},
        {"name": "email", "label": "Email", "field_type": {"type": "email"}, "display_order": 4},
        {"name": "phone", "label": "Phone", "field_type": {"type": "phone"}, "display_order": 5},
        {"name": "category", "label": "Category", "field_type": {"type": "enum", "values": ["raw_materials", "finished_goods", "services", "equipment", "software", "other"]}, "display_order": 6},
        {"name": "tax_id", "label": "Tax ID", "field_type": {"type": "string", "max_length": 50}, "display_order": 7},
        {"name": "payment_terms", "label": "Default Payment Terms", "field_type": {"type": "enum", "values": ["net_15", "net_30", "net_45", "net_60", "due_on_receipt"]}, "default_value": "\"net_30\"", "display_order": 8},
        {"name": "credit_limit", "label": "Credit Limit", "field_type": {"type": "currency", "code": "USD"}, "display_order": 9},
        {"name": "rating", "label": "Rating", "field_type": {"type": "enum", "values": ["a", "b", "c", "d", "f"]}, "display_order": 10},
        {"name": "address", "label": "Address", "field_type": {"type": "address"}, "display_order": 11},
        {"name": "is_active", "label": "Active", "field_type": {"type": "boolean"}, "default_value": true, "display_order": 12}
    ]'::jsonb,
    true,
    true
);

-- ============================================================================
-- Customer Entity (CRM)
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, workflow, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000007',
    'customers',
    'Customer',
    'Customers',
    'crm_customers',
    'Customer and prospect records',
    '[
        {"name": "customer_number", "label": "Customer Number", "field_type": {"type": "string", "max_length": 50}, "is_required": true, "is_unique": true, "display_order": 1},
        {"name": "name", "label": "Customer Name", "field_type": {"type": "string", "max_length": 200}, "is_required": true, "is_searchable": true, "display_order": 2},
        {"name": "type", "label": "Type", "field_type": {"type": "enum", "values": ["individual", "company", "government", "nonprofit"]}, "display_order": 3},
        {"name": "industry", "label": "Industry", "field_type": {"type": "enum", "values": ["technology", "manufacturing", "retail", "healthcare", "finance", "education", "other"]}, "display_order": 4},
        {"name": "email", "label": "Email", "field_type": {"type": "email"}, "display_order": 5},
        {"name": "phone", "label": "Phone", "field_type": {"type": "phone"}, "display_order": 6},
        {"name": "website", "label": "Website", "field_type": {"type": "url"}, "display_order": 7},
        {"name": "revenue", "label": "Annual Revenue", "field_type": {"type": "currency", "code": "USD"}, "display_order": 8},
        {"name": "employee_count", "label": "Employee Count", "field_type": {"type": "integer", "min": 1}, "display_order": 9},
        {"name": "status", "label": "Status", "field_type": {"type": "enum", "values": ["lead", "prospect", "customer", "churned"]}, "default_value": "\"lead\"", "display_order": 10},
        {"name": "owner_id", "label": "Account Owner", "field_type": {"type": "reference", "entity": "employees"}, "display_order": 11},
        {"name": "address", "label": "Address", "field_type": {"type": "address"}, "display_order": 12}
    ]'::jsonb,
    '{
        "name": "customer_lifecycle",
        "initial_state": "lead",
        "states": [
            {"name": "lead", "label": "Lead", "state_type": "working"},
            {"name": "prospect", "label": "Prospect", "state_type": "working"},
            {"name": "customer", "label": "Customer", "state_type": "working"},
            {"name": "churned", "label": "Churned", "state_type": "final"},
            {"name": "inactive", "label": "Inactive", "state_type": "final"}
        ],
        "transitions": [
            {"from_state": "lead", "to_state": "prospect", "action": "qualify", "action_label": "Qualify Lead"},
            {"from_state": "lead", "to_state": "churned", "action": "disqualify", "action_label": "Disqualify"},
            {"from_state": "prospect", "to_state": "customer", "action": "convert", "action_label": "Convert to Customer"},
            {"from_state": "prospect", "to_state": "lead", "action": "revert", "action_label": "Revert to Lead"},
            {"from_state": "prospect", "to_state": "churned", "action": "lost", "action_label": "Lost Opportunity"},
            {"from_state": "customer", "to_state": "churned", "action": "churn", "action_label": "Mark as Churned"},
            {"from_state": "customer", "to_state": "inactive", "action": "deactivate", "action_label": "Deactivate"}
        ],
        "is_active": true
    }'::jsonb,
    true,
    true
);

-- ============================================================================
-- Project Entity
-- ============================================================================

INSERT INTO _atlas.entities (id, name, label, plural_label, table_name, description, fields, workflow, is_audit_enabled, is_soft_delete)
VALUES (
    '10000000-0000-0000-0000-000000000008',
    'projects',
    'Project',
    'Projects',
    'proj_projects',
    'Project management',
    '[
        {"name": "project_number", "label": "Project Number", "field_type": {"type": "string", "max_length": 50}, "is_required": true, "is_unique": true, "display_order": 1},
        {"name": "name", "label": "Project Name", "field_type": {"type": "string", "max_length": 200}, "is_required": true, "is_searchable": true, "display_order": 2},
        {"name": "description", "label": "Description", "field_type": {"type": "rich_text"}, "display_order": 3},
        {"name": "customer_id", "label": "Customer", "field_type": {"type": "reference", "entity": "customers"}, "display_order": 4},
        {"name": "owner_id", "label": "Project Manager", "field_type": {"type": "reference", "entity": "employees"}, "is_required": true, "display_order": 5},
        {"name": "start_date", "label": "Start Date", "field_type": {"type": "date"}, "is_required": true, "display_order": 6},
        {"name": "end_date", "label": "End Date", "field_type": {"type": "date"}, "display_order": 7},
        {"name": "status", "label": "Status", "field_type": {"type": "enum", "values": ["planning", "active", "on_hold", "completed", "cancelled"]}, "default_value": "\"planning\"", "display_order": 8},
        {"name": "budget", "label": "Budget", "field_type": {"type": "currency", "code": "USD"}, "display_order": 9},
        {"name": "spent", "label": "Spent", "field_type": {"type": "currency", "code": "USD"}, "is_read_only": true, "display_order": 10},
        {"name": "progress", "label": "Progress %", "field_type": {"type": "integer", "min": 0, "max": 100}, "default_value": 0, "display_order": 11}
    ]'::jsonb,
    '{
        "name": "project_lifecycle",
        "initial_state": "planning",
        "states": [
            {"name": "planning", "label": "Planning", "state_type": "initial"},
            {"name": "active", "label": "Active", "state_type": "working"},
            {"name": "on_hold", "label": "On Hold", "state_type": "working"},
            {"name": "completed", "label": "Completed", "state_type": "final"},
            {"name": "cancelled", "label": "Cancelled", "state_type": "final"}
        ],
        "transitions": [
            {"from_state": "planning", "to_state": "active", "action": "activate", "action_label": "Start Project"},
            {"from_state": "planning", "to_state": "cancelled", "action": "cancel", "action_label": "Cancel"},
            {"from_state": "active", "to_state": "on_hold", "action": "hold", "action_label": "Put on Hold"},
            {"from_state": "active", "to_state": "completed", "action": "complete", "action_label": "Mark Complete"},
            {"from_state": "on_hold", "to_state": "active", "action": "resume", "action_label": "Resume"},
            {"from_state": "on_hold", "to_state": "cancelled", "action": "cancel", "action_label": "Cancel"}
        ],
        "is_active": true
    }'::jsonb,
    true,
    true
);

-- ============================================================================
-- Initialize Configuration Versions
-- ============================================================================

INSERT INTO _atlas.config_versions (entity_name, version, config)
SELECT name, 1, '{"initialized": true}'::jsonb
FROM _atlas.entities;
