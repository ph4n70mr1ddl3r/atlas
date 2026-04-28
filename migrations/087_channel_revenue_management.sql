-- Channel Revenue Management (Trade Promotion Management)
-- Oracle Fusion Cloud CX > Channel Revenue Management
-- Provides: trade promotions, fund management, claims/settlements, accruals, and analytics

-- Trade promotions (campaigns with channel partners)
CREATE TABLE IF NOT EXISTS _atlas.trade_promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    promotion_number VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    promotion_type VARCHAR(50) NOT NULL, -- billback, off_invoice, lump_sum, volume_tier, fixed_amount
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    priority VARCHAR(20), -- high, medium, low
    category VARCHAR(100),
    partner_id UUID,
    partner_number VARCHAR(50),
    partner_name VARCHAR(300),
    fund_id UUID,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    sell_in_start_date DATE,
    sell_in_end_date DATE,
    sell_out_start_date DATE,
    sell_out_end_date DATE,
    product_category VARCHAR(100),
    product_id UUID,
    product_number VARCHAR(50),
    product_name VARCHAR(300),
    customer_segment VARCHAR(100),
    territory VARCHAR(100),
    expected_revenue NUMERIC(18,2) DEFAULT 0,
    planned_budget NUMERIC(18,2) DEFAULT 0,
    actual_spend NUMERIC(18,2) DEFAULT 0,
    accrued_amount NUMERIC(18,2) DEFAULT 0,
    claimed_amount NUMERIC(18,2) DEFAULT 0,
    settled_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    discount_pct NUMERIC(5,2),
    discount_amount NUMERIC(18,2),
    volume_threshold NUMERIC(18,2),
    volume_uom VARCHAR(30),
    tier_config JSONB DEFAULT '{}',
    objectives TEXT,
    terms_and_conditions TEXT,
    approval_status VARCHAR(30) DEFAULT 'not_submitted',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    owner_id UUID,
    owner_name VARCHAR(200),
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, promotion_number)
);

-- Trade promotion lines (product-level details)
CREATE TABLE IF NOT EXISTS _atlas.trade_promotion_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    promotion_id UUID NOT NULL REFERENCES _atlas.trade_promotions(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    product_id UUID,
    product_number VARCHAR(50),
    product_name VARCHAR(300),
    product_category VARCHAR(100),
    discount_type VARCHAR(30) NOT NULL, -- percentage, fixed_amount, buy_x_get_y
    discount_value NUMERIC(18,2) NOT NULL,
    unit_of_measure VARCHAR(30),
    quantity_from NUMERIC(18,2),
    quantity_to NUMERIC(18,2),
    planned_quantity NUMERIC(18,2) DEFAULT 0,
    actual_quantity NUMERIC(18,2) DEFAULT 0,
    planned_amount NUMERIC(18,2) DEFAULT 0,
    actual_amount NUMERIC(18,2) DEFAULT 0,
    accrual_amount NUMERIC(18,2) DEFAULT 0,
    status VARCHAR(30) DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Promotion funds (budgets allocated for channel activities)
CREATE TABLE IF NOT EXISTS _atlas.promotion_funds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    fund_number VARCHAR(50) NOT NULL,
    name VARCHAR(300) NOT NULL,
    description TEXT,
    fund_type VARCHAR(30) NOT NULL, -- marketing_development, cooperative, market_growth, discretionary
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    partner_id UUID,
    partner_number VARCHAR(50),
    partner_name VARCHAR(300),
    total_budget NUMERIC(18,2) NOT NULL DEFAULT 0,
    allocated_amount NUMERIC(18,2) DEFAULT 0,
    committed_amount NUMERIC(18,2) DEFAULT 0,
    utilized_amount NUMERIC(18,2) DEFAULT 0,
    available_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    fund_year INT,
    fund_quarter VARCHAR(10),
    start_date DATE,
    end_date DATE,
    owner_id UUID,
    owner_name VARCHAR(200),
    approval_status VARCHAR(30) DEFAULT 'not_submitted',
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, fund_number)
);

-- Trade claims (channel partners submit claims for promotions)
CREATE TABLE IF NOT EXISTS _atlas.trade_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    claim_number VARCHAR(50) NOT NULL,
    promotion_id UUID REFERENCES _atlas.trade_promotions(id),
    promotion_number VARCHAR(50),
    fund_id UUID REFERENCES _atlas.promotion_funds(id),
    fund_number VARCHAR(50),
    claim_type VARCHAR(30) NOT NULL, -- billback, proof_of_performance, lump_sum, accrual_adjustment
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    priority VARCHAR(20),
    partner_id UUID,
    partner_number VARCHAR(50),
    partner_name VARCHAR(300),
    claim_date DATE NOT NULL,
    sell_in_from DATE,
    sell_in_to DATE,
    product_id UUID,
    product_number VARCHAR(50),
    product_name VARCHAR(300),
    quantity NUMERIC(18,2) DEFAULT 0,
    unit_of_measure VARCHAR(30),
    unit_price NUMERIC(18,2),
    claimed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    approved_amount NUMERIC(18,2) DEFAULT 0,
    paid_amount NUMERIC(18,2) DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    invoice_number VARCHAR(50),
    invoice_date DATE,
    reference_document VARCHAR(200),
    proof_of_performance JSONB DEFAULT '{}',
    rejection_reason TEXT,
    resolution_notes TEXT,
    assigned_to UUID,
    assigned_to_name VARCHAR(200),
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, claim_number)
);

-- Settlements (payments to channel partners)
CREATE TABLE IF NOT EXISTS _atlas.trade_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    settlement_number VARCHAR(50) NOT NULL,
    claim_id UUID REFERENCES _atlas.trade_claims(id),
    claim_number VARCHAR(50),
    promotion_id UUID REFERENCES _atlas.trade_promotions(id),
    promotion_number VARCHAR(50),
    partner_id UUID,
    partner_number VARCHAR(50),
    partner_name VARCHAR(300),
    settlement_type VARCHAR(30) NOT NULL, -- payment, credit_memo, offset, write_off
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    settlement_date DATE NOT NULL,
    settlement_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    payment_method VARCHAR(30), -- check, wire, ach, credit_note
    payment_reference VARCHAR(100),
    bank_account VARCHAR(100),
    gl_account VARCHAR(50),
    cost_center VARCHAR(50),
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, settlement_number)
);

-- Create indexes
CREATE INDEX idx_trade_promotions_org ON _atlas.trade_promotions(organization_id);
CREATE INDEX idx_trade_promotions_status ON _atlas.trade_promotions(status);
CREATE INDEX idx_trade_promotions_partner ON _atlas.trade_promotions(partner_id);
CREATE INDEX idx_trade_promotions_dates ON _atlas.trade_promotions(start_date, end_date);
CREATE INDEX idx_trade_promotion_lines_promo ON _atlas.trade_promotion_lines(promotion_id);
CREATE INDEX idx_promotion_funds_org ON _atlas.promotion_funds(organization_id);
CREATE INDEX idx_promotion_funds_status ON _atlas.promotion_funds(status);
CREATE INDEX idx_trade_claims_org ON _atlas.trade_claims(organization_id);
CREATE INDEX idx_trade_claims_status ON _atlas.trade_claims(status);
CREATE INDEX idx_trade_claims_promotion ON _atlas.trade_claims(promotion_id);
CREATE INDEX idx_trade_settlements_org ON _atlas.trade_settlements(organization_id);
CREATE INDEX idx_trade_settlements_claim ON _atlas.trade_settlements(claim_id);
