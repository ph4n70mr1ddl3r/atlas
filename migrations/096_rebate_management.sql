-- Rebate Management (Oracle Fusion Cloud: Trade Management > Rebates)
-- Manages supplier and customer rebate agreements, tiers, transactions, accruals, and settlements.

BEGIN;

-- Rebate Agreements
CREATE TABLE IF NOT EXISTS _atlas.rebate_agreements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_number VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    rebate_type VARCHAR(50) NOT NULL, -- supplier_rebate, customer_rebate
    direction VARCHAR(50) NOT NULL, -- receivable (customer pays us), payable (we pay supplier)
    partner_type VARCHAR(50) NOT NULL, -- supplier, customer
    partner_id UUID,
    partner_name VARCHAR(500),
    partner_number VARCHAR(100),
    product_category VARCHAR(200),
    product_id UUID,
    product_name VARCHAR(500),
    uom VARCHAR(50), -- unit of measure for volume tracking
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, active, on_hold, expired, terminated
    calculation_method VARCHAR(50) NOT NULL DEFAULT 'tiered', -- flat_rate, tiered, cumulative
    accrual_account VARCHAR(200),
    liability_account VARCHAR(200),
    expense_account VARCHAR(200),
    payment_terms VARCHAR(100),
    settlement_frequency VARCHAR(50), -- monthly, quarterly, annually, at_end
    minimum_amount DOUBLE PRECISION DEFAULT 0,
    maximum_amount DOUBLE PRECISION,
    auto_accrue BOOLEAN DEFAULT true,
    requires_approval BOOLEAN DEFAULT true,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, agreement_number)
);

-- Rebate Tiers (for tiered rebate agreements)
CREATE TABLE IF NOT EXISTS _atlas.rebate_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_id UUID NOT NULL REFERENCES _atlas.rebate_agreements(id) ON DELETE CASCADE,
    tier_number INT NOT NULL,
    from_value DOUBLE PRECISION NOT NULL DEFAULT 0, -- min quantity or amount
    to_value DOUBLE PRECISION, -- max quantity or amount (NULL = unlimited)
    rebate_rate DOUBLE PRECISION NOT NULL, -- percentage or fixed amount
    rate_type VARCHAR(50) NOT NULL DEFAULT 'percentage', -- percentage, fixed_per_unit, fixed_amount
    description TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Rebate Transactions (qualifying sales/purchases)
CREATE TABLE IF NOT EXISTS _atlas.rebate_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_id UUID NOT NULL REFERENCES _atlas.rebate_agreements(id),
    transaction_number VARCHAR(100) NOT NULL,
    source_type VARCHAR(50), -- sales_order, purchase_order, invoice, manual
    source_id UUID,
    source_number VARCHAR(100),
    transaction_date DATE NOT NULL,
    product_id UUID,
    product_name VARCHAR(500),
    quantity DOUBLE PRECISION DEFAULT 0,
    unit_price DOUBLE PRECISION DEFAULT 0,
    transaction_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) DEFAULT 'USD',
    applicable_rate DOUBLE PRECISION DEFAULT 0,
    rebate_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'eligible', -- eligible, accrued, settled, excluded, disputed
    tier_id UUID,
    excluded_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, transaction_number)
);

-- Rebate Accruals
CREATE TABLE IF NOT EXISTS _atlas.rebate_accruals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_id UUID NOT NULL REFERENCES _atlas.rebate_agreements(id),
    accrual_number VARCHAR(100) NOT NULL,
    accrual_date DATE NOT NULL,
    accrual_period VARCHAR(50), -- e.g., 2024-Q1, 2024-01
    accumulated_quantity DOUBLE PRECISION DEFAULT 0,
    accumulated_amount DOUBLE PRECISION DEFAULT 0,
    applicable_tier_id UUID,
    applicable_rate DOUBLE PRECISION DEFAULT 0,
    accrued_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) DEFAULT 'USD',
    gl_posted BOOLEAN DEFAULT false,
    gl_journal_id UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, posted, reversed, settled
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, accrual_number)
);

-- Rebate Settlements (actual payments/receipts)
CREATE TABLE IF NOT EXISTS _atlas.rebate_settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agreement_id UUID NOT NULL REFERENCES _atlas.rebate_agreements(id),
    settlement_number VARCHAR(100) NOT NULL,
    settlement_date DATE NOT NULL,
    settlement_period_from DATE,
    settlement_period_to DATE,
    total_qualifying_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_qualifying_quantity DOUBLE PRECISION DEFAULT 0,
    applicable_tier_id UUID,
    applicable_rate DOUBLE PRECISION DEFAULT 0,
    settlement_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) DEFAULT 'USD',
    settlement_type VARCHAR(50) NOT NULL DEFAULT 'payment', -- payment, credit_memo, offset
    payment_method VARCHAR(50), -- check, wire, ach, credit_note
    payment_reference VARCHAR(200),
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, approved, paid, cancelled, disputed
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, settlement_number)
);

-- Settlement Lines (link to specific transactions)
CREATE TABLE IF NOT EXISTS _atlas.rebate_settlement_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id UUID NOT NULL REFERENCES _atlas.rebate_settlements(id) ON DELETE CASCADE,
    transaction_id UUID NOT NULL REFERENCES _atlas.rebate_transactions(id),
    settlement_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_rebate_agreements_org ON _atlas.rebate_agreements(organization_id);
CREATE INDEX IF NOT EXISTS idx_rebate_agreements_status ON _atlas.rebate_agreements(status);
CREATE INDEX IF NOT EXISTS idx_rebate_agreements_partner ON _atlas.rebate_agreements(partner_id);
CREATE INDEX IF NOT EXISTS idx_rebate_tiers_agreement ON _atlas.rebate_tiers(agreement_id);
CREATE INDEX IF NOT EXISTS idx_rebate_transactions_agreement ON _atlas.rebate_transactions(agreement_id);
CREATE INDEX IF NOT EXISTS idx_rebate_transactions_status ON _atlas.rebate_transactions(status);
CREATE INDEX IF NOT EXISTS idx_rebate_accruals_agreement ON _atlas.rebate_accruals(agreement_id);
CREATE INDEX IF NOT EXISTS idx_rebate_accruals_status ON _atlas.rebate_accruals(status);
CREATE INDEX IF NOT EXISTS idx_rebate_settlements_agreement ON _atlas.rebate_settlements(agreement_id);
CREATE INDEX IF NOT EXISTS idx_rebate_settlements_status ON _atlas.rebate_settlements(status);
CREATE INDEX IF NOT EXISTS idx_rebate_settlement_lines_settlement ON _atlas.rebate_settlement_lines(settlement_id);

COMMIT;
