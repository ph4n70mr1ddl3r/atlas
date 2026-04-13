-- Atlas ERP - Create actual data tables for seeded entities
-- These tables back the CRUD operations for domain entities

-- ============================================================================
-- HCM Tables
-- ============================================================================

CREATE TABLE IF NOT EXISTS hcm_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    parent_id UUID REFERENCES hcm_departments(id),
    cost_center TEXT,
    is_active BOOLEAN DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_hcm_departments_code ON hcm_departments(code);

CREATE TABLE IF NOT EXISTS hcm_employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'active',
    employee_number TEXT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    middle_name TEXT,
    email VARCHAR(255),
    phone VARCHAR(50),
    date_of_birth DATE,
    hire_date DATE,
    termination_date DATE,
    department_id UUID REFERENCES hcm_departments(id),
    position_id UUID,
    manager_id UUID REFERENCES hcm_employees(id),
    employment_type VARCHAR(50) DEFAULT 'full_time',
    status VARCHAR(50) DEFAULT 'active',
    address JSONB
);

CREATE INDEX IF NOT EXISTS idx_hcm_employees_email ON hcm_employees(email);
CREATE INDEX IF NOT EXISTS idx_hcm_employees_dept ON hcm_employees(department_id);

-- ============================================================================
-- SCM Tables
-- ============================================================================

CREATE TABLE IF NOT EXISTS scm_suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    supplier_number TEXT NOT NULL,
    name TEXT NOT NULL,
    contact_name TEXT,
    email VARCHAR(255),
    phone VARCHAR(50),
    category VARCHAR(50),
    tax_id TEXT,
    payment_terms VARCHAR(50) DEFAULT 'net_30',
    credit_limit NUMERIC(18,2),
    rating VARCHAR(10),
    address JSONB,
    is_active BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS scm_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    sku TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    product_type VARCHAR(50),
    category VARCHAR(50),
    supplier_id UUID REFERENCES scm_suppliers(id),
    unit_price NUMERIC(18,2),
    cost_price NUMERIC(18,2),
    reorder_level INTEGER,
    reorder_quantity INTEGER,
    unit_of_measure TEXT,
    weight NUMERIC(10,3),
    is_active BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS scm_warehouses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    address JSONB,
    manager_id UUID,
    is_active BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS scm_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    product_id UUID REFERENCES scm_products(id),
    warehouse_id UUID REFERENCES scm_warehouses(id),
    quantity_on_hand INTEGER DEFAULT 0,
    quantity_reserved INTEGER DEFAULT 0,
    quantity_available INTEGER DEFAULT 0,
    unit_cost NUMERIC(18,2),
    location_code TEXT,
    last_counted_date DATE
);

CREATE TABLE IF NOT EXISTS scm_purchase_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    po_number TEXT NOT NULL,
    supplier_id UUID REFERENCES scm_suppliers(id),
    order_date DATE,
    expected_date DATE,
    currency_code VARCHAR(10) DEFAULT 'USD',
    subtotal NUMERIC(18,2),
    tax_amount NUMERIC(18,2),
    total_amount NUMERIC(18,2),
    payment_terms VARCHAR(50),
    shipping_address JSONB,
    notes TEXT,
    approved_by UUID
);

CREATE TABLE IF NOT EXISTS scm_sales_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    order_number TEXT NOT NULL,
    customer_id UUID,
    order_date DATE,
    expected_delivery DATE,
    subtotal NUMERIC(18,2),
    tax NUMERIC(18,2),
    total NUMERIC(18,2),
    priority VARCHAR(20) DEFAULT 'normal',
    sales_rep_id UUID
);

-- ============================================================================
-- CRM Tables
-- ============================================================================

CREATE TABLE IF NOT EXISTS crm_customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'lead',
    customer_number TEXT NOT NULL,
    name TEXT NOT NULL,
    type VARCHAR(50) DEFAULT 'company',
    industry VARCHAR(50),
    email VARCHAR(255),
    phone VARCHAR(50),
    website VARCHAR(2048),
    revenue NUMERIC(18,2),
    employee_count INTEGER,
    status VARCHAR(50) DEFAULT 'lead',
    owner_id UUID,
    address JSONB
);

CREATE TABLE IF NOT EXISTS crm_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'new',
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    company TEXT,
    title TEXT,
    source VARCHAR(50),
    industry VARCHAR(50),
    estimated_value NUMERIC(18,2),
    assigned_to_id UUID,
    customer_id UUID,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS crm_opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'discovery',
    name TEXT NOT NULL,
    customer_id UUID,
    lead_id UUID,
    amount NUMERIC(18,2),
    probability INTEGER,
    expected_close_date DATE,
    stage VARCHAR(50),
    assigned_to_id UUID,
    contact_id UUID
);

CREATE TABLE IF NOT EXISTS crm_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    title TEXT,
    customer_id UUID,
    preferred_contact VARCHAR(50) DEFAULT 'email',
    is_primary BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS crm_service_cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'open',
    subject TEXT NOT NULL,
    description TEXT,
    customer_id UUID,
    contact_id UUID,
    priority VARCHAR(20) DEFAULT 'medium',
    category VARCHAR(50),
    assigned_to_id UUID
);

-- ============================================================================
-- Financials Tables
-- ============================================================================

CREATE TABLE IF NOT EXISTS fin_chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    account_number TEXT NOT NULL,
    name TEXT NOT NULL,
    account_type VARCHAR(50) NOT NULL,
    subtype VARCHAR(50),
    parent_account_id UUID REFERENCES fin_chart_of_accounts(id),
    is_active BOOLEAN DEFAULT true,
    description TEXT
);

CREATE TABLE IF NOT EXISTS fin_journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    entry_number TEXT NOT NULL,
    entry_date DATE,
    description TEXT,
    entry_type VARCHAR(50) DEFAULT 'standard',
    total_debit NUMERIC(18,2),
    total_credit NUMERIC(18,2),
    created_by_id UUID,
    is_posted BOOLEAN DEFAULT false
);

CREATE TABLE IF NOT EXISTS fin_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    invoice_number TEXT NOT NULL,
    customer_id UUID,
    invoice_date DATE,
    due_date DATE,
    status VARCHAR(50) DEFAULT 'draft',
    subtotal NUMERIC(18,2),
    tax_amount NUMERIC(18,2),
    total_amount NUMERIC(18,2),
    amount_paid NUMERIC(18,2) DEFAULT 0,
    balance_due NUMERIC(18,2),
    payment_terms VARCHAR(50) DEFAULT 'net_30',
    notes TEXT
);

CREATE TABLE IF NOT EXISTS fin_budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    name TEXT NOT NULL,
    department_id UUID,
    period VARCHAR(50),
    start_date DATE,
    end_date DATE,
    total_budget NUMERIC(18,2),
    allocated NUMERIC(18,2),
    spent NUMERIC(18,2),
    owner_id UUID
);

-- ============================================================================
-- Projects Tables
-- ============================================================================

CREATE TABLE IF NOT EXISTS proj_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'planning',
    project_number TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    customer_id UUID,
    owner_id UUID,
    start_date DATE,
    end_date DATE,
    budget NUMERIC(18,2),
    spent NUMERIC(18,2) DEFAULT 0,
    progress INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS proj_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'todo',
    title TEXT NOT NULL,
    description TEXT,
    project_id UUID REFERENCES proj_projects(id),
    assignee_id UUID,
    parent_task_id UUID,
    priority VARCHAR(20) DEFAULT 'medium',
    task_type VARCHAR(50) DEFAULT 'task',
    estimated_hours INTEGER,
    actual_hours INTEGER,
    due_date DATE,
    start_date DATE,
    progress INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS proj_timesheets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    workflow_state VARCHAR(100) DEFAULT 'draft',
    employee_id UUID,
    project_id UUID REFERENCES proj_projects(id),
    task_id UUID,
    date DATE,
    hours NUMERIC(5,2),
    description TEXT,
    entry_type VARCHAR(50) DEFAULT 'regular',
    billable BOOLEAN DEFAULT true
);

CREATE TABLE IF NOT EXISTS proj_milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    name TEXT NOT NULL,
    project_id UUID REFERENCES proj_projects(id),
    target_date DATE,
    actual_date DATE,
    status VARCHAR(50) DEFAULT 'planned',
    budget NUMERIC(18,2),
    description TEXT
);

-- ============================================================================
-- Indexes
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_crm_customers_number ON crm_customers(customer_number);
CREATE INDEX IF NOT EXISTS idx_crm_leads_email ON crm_leads(email);
CREATE INDEX IF NOT EXISTS idx_scm_products_sku ON scm_products(sku);
CREATE INDEX IF NOT EXISTS idx_scm_purchase_orders_number ON scm_purchase_orders(po_number);
CREATE INDEX IF NOT EXISTS idx_proj_projects_number ON proj_projects(project_number);
CREATE INDEX IF NOT EXISTS idx_proj_tasks_project ON proj_tasks(project_id);
