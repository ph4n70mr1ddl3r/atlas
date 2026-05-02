-- 110_advance_payment_customer_deposit_cash_position.sql
-- Oracle Fusion Financial Features: Advance Payments, Customer Deposits, Cash Position

-- ============================================================================
-- Advance Payments (Oracle Fusion: AP > Advance Payments / Prepayments)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.advance_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    advance_number VARCHAR(50) NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(300) NOT NULL,
    supplier_site_id UUID,
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    currency_code VARCHAR(3) NOT NULL,
    advance_amount NUMERIC(18,2) NOT NULL,
    applied_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    unapplied_amount NUMERIC(18,2) NOT NULL,
    exchange_rate NUMERIC(18,6),
    payment_method VARCHAR(30),
    payment_reference VARCHAR(100),
    prepayment_account_code VARCHAR(50),
    liability_account_code VARCHAR(50),
    advance_date DATE NOT NULL,
    payment_date DATE,
    due_date DATE,
    expiration_date DATE,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    paid_by UUID,
    paid_at TIMESTAMPTZ,
    cancelled_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, advance_number)
);

CREATE INDEX idx_advance_payments_org ON financials.advance_payments(organization_id);
CREATE INDEX idx_advance_payments_supplier ON financials.advance_payments(supplier_id);
CREATE INDEX idx_advance_payments_status ON financials.advance_payments(status);
CREATE INDEX idx_advance_payments_date ON financials.advance_payments(advance_date);

CREATE TABLE IF NOT EXISTS financials.advance_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    advance_id UUID NOT NULL REFERENCES financials.advance_payments(id),
    advance_number VARCHAR(50),
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50),
    applied_amount NUMERIC(18,2) NOT NULL,
    application_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'applied',
    gl_account_code VARCHAR(50),
    reversed_at TIMESTAMPTZ,
    reversed_by UUID,
    reversal_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    applied_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_advance_applications_advance ON financials.advance_applications(advance_id);
CREATE INDEX idx_advance_applications_invoice ON financials.advance_applications(invoice_id);
CREATE INDEX idx_advance_applications_status ON financials.advance_applications(status);

-- ============================================================================
-- Customer Deposits (Oracle Fusion: AR > Customer Deposits)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.customer_deposits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    deposit_number VARCHAR(50) NOT NULL,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(300) NOT NULL,
    customer_site_id UUID,
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    currency_code VARCHAR(3) NOT NULL,
    deposit_amount NUMERIC(18,2) NOT NULL,
    applied_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    unapplied_amount NUMERIC(18,2) NOT NULL,
    exchange_rate NUMERIC(18,6),
    deposit_account_code VARCHAR(50),
    receivable_account_code VARCHAR(50),
    deposit_date DATE NOT NULL,
    receipt_date DATE,
    receipt_reference VARCHAR(100),
    expiration_date DATE,
    received_by UUID,
    received_at TIMESTAMPTZ,
    refund_reference VARCHAR(100),
    refunded_by UUID,
    refunded_at TIMESTAMPTZ,
    cancelled_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, deposit_number)
);

CREATE INDEX idx_customer_deposits_org ON financials.customer_deposits(organization_id);
CREATE INDEX idx_customer_deposits_customer ON financials.customer_deposits(customer_id);
CREATE INDEX idx_customer_deposits_status ON financials.customer_deposits(status);
CREATE INDEX idx_customer_deposits_date ON financials.customer_deposits(deposit_date);

CREATE TABLE IF NOT EXISTS financials.deposit_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    deposit_id UUID NOT NULL REFERENCES financials.customer_deposits(id),
    deposit_number VARCHAR(50),
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50),
    applied_amount NUMERIC(18,2) NOT NULL,
    application_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'applied',
    gl_account_code VARCHAR(50),
    reversed_at TIMESTAMPTZ,
    reversed_by UUID,
    reversal_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    applied_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_deposit_applications_deposit ON financials.deposit_applications(deposit_id);
CREATE INDEX idx_deposit_applications_invoice ON financials.deposit_applications(invoice_id);
CREATE INDEX idx_deposit_applications_status ON financials.deposit_applications(status);

-- ============================================================================
-- Cash Position (Oracle Fusion: Treasury > Cash Position)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.cash_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    bank_account_id UUID NOT NULL,
    bank_account_number VARCHAR(50),
    bank_account_name VARCHAR(200),
    currency_code VARCHAR(3) NOT NULL,
    opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_inflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_outflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    ledger_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    available_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    hold_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    position_date DATE NOT NULL,
    source_breakdown JSONB DEFAULT '{}'::jsonb,
    metadata JSONB DEFAULT '{}'::jsonb,
    calculated_at TIMESTAMPTZ DEFAULT now(),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_cash_positions_org ON financials.cash_positions(organization_id);
CREATE INDEX idx_cash_positions_account ON financials.cash_positions(bank_account_id);
CREATE INDEX idx_cash_positions_date ON financials.cash_positions(position_date);
CREATE INDEX idx_cash_positions_currency ON financials.cash_positions(currency_code);
CREATE UNIQUE INDEX idx_cash_positions_unique ON financials.cash_positions(organization_id, bank_account_id, position_date);

CREATE TABLE IF NOT EXISTS financials.cash_position_summaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    currency_code VARCHAR(3) NOT NULL,
    position_date DATE NOT NULL,
    total_opening_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_inflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_outflows NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_closing_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_ledger_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_available_balance NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_hold_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    account_count INT NOT NULL DEFAULT 0,
    accounts JSONB DEFAULT '[]'::jsonb,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_cash_pos_summaries_org ON financials.cash_position_summaries(organization_id);
CREATE INDEX idx_cash_pos_summaries_date ON financials.cash_position_summaries(position_date);
CREATE INDEX idx_cash_pos_summaries_currency ON financials.cash_position_summaries(currency_code);
CREATE UNIQUE INDEX idx_cash_pos_summaries_unique ON financials.cash_position_summaries(organization_id, currency_code, position_date);
