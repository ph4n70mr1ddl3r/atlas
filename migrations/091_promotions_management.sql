-- Promotions Management (Oracle Fusion Trade Management > Trade Promotion)
-- Tables for trade promotions, promotional offers, fund allocations, claims, and ROI tracking.

CREATE TABLE IF NOT EXISTS _atlas.promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    promotion_type VARCHAR(50) NOT NULL DEFAULT 'trade',
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(200),
    territory_id UUID,
    product_id UUID,
    product_name VARCHAR(200),
    budget_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    spent_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    owner_id UUID,
    owner_name VARCHAR(200),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_promotions_org ON _atlas.promotions(organization_id);
CREATE INDEX IF NOT EXISTS idx_promotions_status ON _atlas.promotions(status);
CREATE INDEX IF NOT EXISTS idx_promotions_type ON _atlas.promotions(promotion_type);
CREATE INDEX IF NOT EXISTS idx_promotions_dates ON _atlas.promotions(start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_promotions_customer ON _atlas.promotions(customer_id);

CREATE TABLE IF NOT EXISTS _atlas.promotion_offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    promotion_id UUID NOT NULL REFERENCES _atlas.promotions(id) ON DELETE CASCADE,
    offer_type VARCHAR(50) NOT NULL,
    description TEXT,
    discount_type VARCHAR(50) NOT NULL DEFAULT 'percentage',
    discount_value NUMERIC(18,2) NOT NULL DEFAULT 0,
    buy_quantity INT,
    get_quantity INT,
    minimum_purchase NUMERIC(18,2),
    maximum_discount NUMERIC(18,2),
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_promotion_offers_promo ON _atlas.promotion_offers(promotion_id);
CREATE INDEX IF NOT EXISTS idx_promotion_offers_type ON _atlas.promotion_offers(offer_type);

CREATE TABLE IF NOT EXISTS _atlas.promotion_funds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    promotion_id UUID NOT NULL REFERENCES _atlas.promotions(id) ON DELETE CASCADE,
    fund_type VARCHAR(50) NOT NULL,
    allocated_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    committed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    spent_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_promotion_funds_promo ON _atlas.promotion_funds(promotion_id);

CREATE TABLE IF NOT EXISTS _atlas.promotion_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    promotion_id UUID NOT NULL REFERENCES _atlas.promotions(id) ON DELETE CASCADE,
    claim_number VARCHAR(50) NOT NULL,
    claim_type VARCHAR(50) NOT NULL DEFAULT 'accrual',
    status VARCHAR(50) NOT NULL DEFAULT 'submitted',
    amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    approved_amount NUMERIC(18,2),
    paid_amount NUMERIC(18,2),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    claim_date DATE NOT NULL,
    settlement_date DATE,
    customer_id UUID,
    customer_name VARCHAR(200),
    description TEXT,
    rejection_reason TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_promotion_claims_promo ON _atlas.promotion_claims(promotion_id);
CREATE INDEX IF NOT EXISTS idx_promotion_claims_status ON _atlas.promotion_claims(status);
CREATE INDEX IF NOT EXISTS idx_promotion_claims_number ON _atlas.promotion_claims(claim_number);
