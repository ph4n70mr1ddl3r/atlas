-- Lead and Opportunity Management (Oracle Fusion CX Sales)
-- Tables for lead tracking, opportunity pipeline, sales activities,
-- and pipeline analytics.

-- Lead Sources (configurable reference data)
CREATE TABLE IF NOT EXISTS _atlas.lead_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Lead Rating Models (for lead scoring)
CREATE TABLE IF NOT EXISTS _atlas.lead_rating_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    scoring_criteria JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Sales Leads
CREATE TABLE IF NOT EXISTS _atlas.sales_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lead_number VARCHAR(50) NOT NULL,
    first_name VARCHAR(200),
    last_name VARCHAR(200),
    company VARCHAR(500),
    title VARCHAR(200),
    email VARCHAR(500),
    phone VARCHAR(50),
    website VARCHAR(500),
    industry VARCHAR(100),
    lead_source_id UUID REFERENCES _atlas.lead_sources(id),
    lead_source_name VARCHAR(200),
    lead_rating_model_id UUID REFERENCES _atlas.lead_rating_models(id),
    lead_score DOUBLE PRECISION DEFAULT 0,
    lead_rating VARCHAR(20) DEFAULT 'cold',
    estimated_value DOUBLE PRECISION DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(30) DEFAULT 'new',
    owner_id UUID,
    owner_name VARCHAR(200),
    converted_opportunity_id UUID,
    converted_customer_id UUID,
    converted_at TIMESTAMPTZ,
    notes TEXT,
    address JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, lead_number)
);

-- Opportunity Stages (configurable pipeline stages)
CREATE TABLE IF NOT EXISTS _atlas.opportunity_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    probability DOUBLE PRECISION DEFAULT 0,
    display_order INT DEFAULT 0,
    is_won BOOLEAN DEFAULT false,
    is_lost BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    is_closed BOOLEAN GENERATED ALWAYS AS (is_won OR is_lost) STORED,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Sales Opportunities
CREATE TABLE IF NOT EXISTS _atlas.sales_opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    opportunity_number VARCHAR(50) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    customer_id UUID,
    customer_name VARCHAR(500),
    lead_id UUID REFERENCES _atlas.sales_leads(id),
    stage_id UUID REFERENCES _atlas.opportunity_stages(id),
    stage_name VARCHAR(200),
    amount DOUBLE PRECISION DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    probability DOUBLE PRECISION DEFAULT 0,
    weighted_amount DOUBLE PRECISION DEFAULT 0,
    expected_close_date DATE,
    actual_close_date DATE,
    status VARCHAR(30) DEFAULT 'open',
    owner_id UUID,
    owner_name VARCHAR(200),
    contact_id UUID,
    contact_name VARCHAR(200),
    competitor VARCHAR(500),
    lost_reason TEXT,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, opportunity_number)
);

-- Opportunity Line Items
CREATE TABLE IF NOT EXISTS _atlas.opportunity_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    opportunity_id UUID NOT NULL REFERENCES _atlas.sales_opportunities(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    product_name VARCHAR(500) NOT NULL,
    product_code VARCHAR(100),
    description TEXT,
    quantity DOUBLE PRECISION DEFAULT 1,
    unit_price DOUBLE PRECISION DEFAULT 0,
    line_amount DOUBLE PRECISION DEFAULT 0,
    discount_percent DOUBLE PRECISION DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Sales Activities (calls, meetings, tasks associated with leads/opportunities)
CREATE TABLE IF NOT EXISTS _atlas.sales_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    subject VARCHAR(500) NOT NULL,
    description TEXT,
    activity_type VARCHAR(50) NOT NULL,
    status VARCHAR(30) DEFAULT 'planned',
    priority VARCHAR(20) DEFAULT 'medium',
    lead_id UUID REFERENCES _atlas.sales_leads(id),
    opportunity_id UUID REFERENCES _atlas.sales_opportunities(id),
    contact_id UUID,
    contact_name VARCHAR(200),
    owner_id UUID,
    owner_name VARCHAR(200),
    start_at TIMESTAMPTZ,
    end_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    outcome TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Opportunity Stage History (audit trail for stage changes)
CREATE TABLE IF NOT EXISTS _atlas.opportunity_stage_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    opportunity_id UUID NOT NULL REFERENCES _atlas.sales_opportunities(id) ON DELETE CASCADE,
    from_stage VARCHAR(200),
    to_stage VARCHAR(200) NOT NULL,
    changed_by UUID,
    changed_by_name VARCHAR(200),
    changed_at TIMESTAMPTZ DEFAULT now(),
    notes TEXT
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_sales_leads_org ON _atlas.sales_leads(organization_id);
CREATE INDEX IF NOT EXISTS idx_sales_leads_status ON _atlas.sales_leads(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_sales_leads_owner ON _atlas.sales_leads(owner_id);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_org ON _atlas.sales_opportunities(organization_id);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_status ON _atlas.sales_opportunities(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_stage ON _atlas.sales_opportunities(stage_id);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_owner ON _atlas.sales_opportunities(owner_id);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_close_date ON _atlas.sales_opportunities(expected_close_date);
CREATE INDEX IF NOT EXISTS idx_opportunity_lines_opportunity ON _atlas.opportunity_lines(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_sales_activities_lead ON _atlas.sales_activities(lead_id);
CREATE INDEX IF NOT EXISTS idx_sales_activities_opportunity ON _atlas.sales_activities(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_opportunity_stage_history_opportunity ON _atlas.opportunity_stage_history(opportunity_id);
