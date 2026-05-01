-- Tax Reporting & Filing (Oracle Fusion Tax > Tax Reporting)
-- Migration 104

-- Tax return templates define the structure of tax returns
CREATE TABLE IF NOT EXISTS _atlas.tax_return_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    tax_type VARCHAR(50) NOT NULL, -- vat, gst, sales_tax, corporate_income, withholding
    jurisdiction_code VARCHAR(100),
    filing_frequency VARCHAR(30) NOT NULL DEFAULT 'monthly', -- monthly, quarterly, semi_annual, annual
    return_form_number VARCHAR(50),
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Tax return template lines (boxes/fields on the tax return)
CREATE TABLE IF NOT EXISTS _atlas.tax_return_template_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.tax_return_templates(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    box_code VARCHAR(50) NOT NULL,
    box_name VARCHAR(200) NOT NULL,
    description TEXT,
    line_type VARCHAR(50) NOT NULL DEFAULT 'input', -- input, calculated, total, informational
    calculation_formula TEXT,
    account_code_filter VARCHAR(100),
    tax_rate_code_filter VARCHAR(50),
    is_debit BOOLEAN DEFAULT false,
    display_order INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Tax returns (filed returns)
CREATE TABLE IF NOT EXISTS _atlas.tax_returns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    return_number VARCHAR(100) NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.tax_return_templates(id),
    template_name VARCHAR(200),
    tax_type VARCHAR(50),
    jurisdiction_code VARCHAR(100),
    filing_period_start DATE NOT NULL,
    filing_period_end DATE NOT NULL,
    filing_due_date DATE,
    total_tax_amount DECIMAL(18,2) DEFAULT 0,
    total_taxable_amount DECIMAL(18,2) DEFAULT 0,
    total_exempt_amount DECIMAL(18,2) DEFAULT 0,
    total_input_tax DECIMAL(18,2) DEFAULT 0,
    total_output_tax DECIMAL(18,2) DEFAULT 0,
    net_tax_due DECIMAL(18,2) DEFAULT 0,
    penalty_amount DECIMAL(18,2) DEFAULT 0,
    interest_amount DECIMAL(18,2) DEFAULT 0,
    total_amount_due DECIMAL(18,2) DEFAULT 0,
    payment_amount DECIMAL(18,2) DEFAULT 0,
    refund_amount DECIMAL(18,2) DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, submitted, filed, paid, amended, rejected
    filing_method VARCHAR(30), -- electronic, paper
    filing_reference VARCHAR(100),
    filing_date DATE,
    payment_date DATE,
    payment_reference VARCHAR(100),
    amendment_reason TEXT,
    notes TEXT,
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    filed_by UUID,
    filed_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, return_number)
);

-- Tax return line values (populated from template lines + actual data)
CREATE TABLE IF NOT EXISTS _atlas.tax_return_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    tax_return_id UUID NOT NULL REFERENCES _atlas.tax_returns(id) ON DELETE CASCADE,
    template_line_id UUID REFERENCES _atlas.tax_return_template_lines(id),
    line_number INT NOT NULL,
    box_code VARCHAR(50) NOT NULL,
    box_name VARCHAR(200),
    line_type VARCHAR(50) NOT NULL DEFAULT 'input',
    amount DECIMAL(18,2) DEFAULT 0,
    calculated_amount DECIMAL(18,2) DEFAULT 0,
    override_amount DECIMAL(18,2),
    final_amount DECIMAL(18,2) DEFAULT 0,
    description TEXT,
    source_count INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Tax filing calendar
CREATE TABLE IF NOT EXISTS _atlas.tax_filing_calendar (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID NOT NULL REFERENCES _atlas.tax_return_templates(id),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    due_date DATE NOT NULL,
    filing_status VARCHAR(50) NOT NULL DEFAULT 'upcoming', -- upcoming, due_soon, overdue, filed, extended
    return_id UUID REFERENCES _atlas.tax_returns(id),
    extension_filed BOOLEAN DEFAULT false,
    extension_due_date DATE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Tax reporting dashboard summary (materialized view alternative)
CREATE TABLE IF NOT EXISTS _atlas.tax_reporting_dashboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL UNIQUE,
    total_templates INT DEFAULT 0,
    active_templates INT DEFAULT 0,
    total_returns INT DEFAULT 0,
    draft_returns INT DEFAULT 0,
    filed_returns INT DEFAULT 0,
    overdue_returns INT DEFAULT 0,
    total_tax_paid DECIMAL(18,2) DEFAULT 0,
    total_tax_due DECIMAL(18,2) DEFAULT 0,
    total_refunds DECIMAL(18,2) DEFAULT 0,
    upcoming_filings INT DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tax_return_templates_org ON _atlas.tax_return_templates(organization_id);
CREATE INDEX IF NOT EXISTS idx_tax_return_template_lines_template ON _atlas.tax_return_template_lines(template_id);
CREATE INDEX IF NOT EXISTS idx_tax_returns_org ON _atlas.tax_returns(organization_id);
CREATE INDEX IF NOT EXISTS idx_tax_returns_status ON _atlas.tax_returns(status);
CREATE INDEX IF NOT EXISTS idx_tax_return_lines_return ON _atlas.tax_return_lines(tax_return_id);
CREATE INDEX IF NOT EXISTS idx_tax_filing_calendar_org ON _atlas.tax_filing_calendar(organization_id);
CREATE INDEX IF NOT EXISTS idx_tax_filing_calendar_due ON _atlas.tax_filing_calendar(due_date);
