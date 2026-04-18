-- Subscription Management (Oracle Fusion Subscription Management)
-- Manages recurring billing, subscription lifecycles, amendments, and revenue schedules.
-- Oracle Fusion equivalent: Financials > Subscription Management > Subscriptions

-- Subscription products / catalog
CREATE TABLE IF NOT EXISTS _atlas.subscription_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    product_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    product_type VARCHAR(30) NOT NULL DEFAULT 'service', -- service, software, physical, bundle
    billing_frequency VARCHAR(30) NOT NULL DEFAULT 'monthly', -- monthly, quarterly, semi_annual, annual, one_time
    default_duration_months INT NOT NULL DEFAULT 12,
    is_auto_renew BOOLEAN NOT NULL DEFAULT true,
    cancellation_notice_days INT NOT NULL DEFAULT 30,
    setup_fee NUMERIC(18, 2) NOT NULL DEFAULT 0,
    tier_type VARCHAR(20) NOT NULL DEFAULT 'flat', -- flat, volume, tiered, stairstep
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, product_code)
);

-- Subscription product price tiers
CREATE TABLE IF NOT EXISTS _atlas.subscription_price_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    product_id UUID NOT NULL REFERENCES _atlas.subscription_products(id),
    tier_name VARCHAR(100),
    min_quantity NUMERIC(18, 2) NOT NULL DEFAULT 0,
    max_quantity NUMERIC(18, 2),
    unit_price NUMERIC(18, 2) NOT NULL DEFAULT 0,
    discount_percent NUMERIC(5, 2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    effective_from DATE,
    effective_to DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sub_price_tiers_product ON _atlas.subscription_price_tiers(product_id);

-- Subscriptions (main header)
CREATE TABLE IF NOT EXISTS _atlas.subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    subscription_number VARCHAR(50) NOT NULL UNIQUE,
    customer_id UUID NOT NULL,
    customer_name VARCHAR(200),
    product_id UUID NOT NULL REFERENCES _atlas.subscription_products(id),
    product_code VARCHAR(50),
    product_name VARCHAR(200),
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, active, suspended, cancelled, expired, renewed
    start_date DATE NOT NULL,
    end_date DATE,
    renewal_date DATE,
    billing_frequency VARCHAR(30) NOT NULL DEFAULT 'monthly',
    billing_dayOfMonth INT NOT NULL DEFAULT 1,
    billing_alignment VARCHAR(20) NOT NULL DEFAULT 'start_date', -- start_date, first_of_month, anniversary
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    quantity NUMERIC(18, 2) NOT NULL DEFAULT 1,
    unit_price NUMERIC(18, 2) NOT NULL DEFAULT 0,
    list_price NUMERIC(18, 2) NOT NULL DEFAULT 0,
    discount_percent NUMERIC(5, 2) NOT NULL DEFAULT 0,
    setup_fee NUMERIC(18, 2) NOT NULL DEFAULT 0,
    recurring_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_contract_value NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_billed NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_revenue recognized NUMERIC(18, 2) NOT NULL DEFAULT 0,
    duration_months INT NOT NULL DEFAULT 12,
    is_auto_renew BOOLEAN NOT NULL DEFAULT true,
    cancellation_date DATE,
    cancellation_reason TEXT,
    suspension_reason TEXT,
    sales_rep_id UUID,
    sales_rep_name VARCHAR(200),
    gl_revenue_account VARCHAR(50),
    gl_deferred_account VARCHAR(50),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_subscriptions_org ON _atlas.subscriptions(organization_id);
CREATE INDEX idx_subscriptions_customer ON _atlas.subscriptions(customer_id);
CREATE INDEX idx_subscriptions_product ON _atlas.subscriptions(product_id);
CREATE INDEX idx_subscriptions_status ON _atlas.subscriptions(organization_id, status);
CREATE INDEX idx_subscriptions_renewal ON _atlas.subscriptions(renewal_date) WHERE status = 'active' AND is_auto_renew = true;

-- Subscription amendments (changes to active subscriptions)
CREATE TABLE IF NOT EXISTS _atlas.subscription_amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    subscription_id UUID NOT NULL REFERENCES _atlas.subscriptions(id),
    amendment_number VARCHAR(50) NOT NULL,
    amendment_type VARCHAR(30) NOT NULL, -- price_change, quantity_change, upgrade, downgrade, renewal, cancellation, suspension, reactivation
    description TEXT,
    old_quantity NUMERIC(18, 2),
    new_quantity NUMERIC(18, 2),
    old_unit_price NUMERIC(18, 2),
    new_unit_price NUMERIC(18, 2),
    old_recurring_amount NUMERIC(18, 2),
    new_recurring_amount NUMERIC(18, 2),
    old_end_date DATE,
    new_end_date DATE,
    effective_date DATE NOT NULL,
    proration_credit NUMERIC(18, 2) DEFAULT 0,
    proration_charge NUMERIC(18, 2) DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'draft', -- draft, applied, cancelled
    applied_at TIMESTAMPTZ,
    applied_by UUID,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(subscription_id, amendment_number)
);

CREATE INDEX idx_sub_amendments_subscription ON _atlas.subscription_amendments(subscription_id);
CREATE INDEX idx_sub_amendments_status ON _atlas.subscription_amendments(subscription_id, status);

-- Subscription billing schedule
CREATE TABLE IF NOT EXISTS _atlas.subscription_billing_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    subscription_id UUID NOT NULL REFERENCES _atlas.subscriptions(id),
    schedule_number INT NOT NULL,
    billing_date DATE NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    proration_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    invoice_id UUID,
    invoice_number VARCHAR(50),
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- pending, invoiced, paid, cancelled, skipped
    paid_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sub_billing_subscription ON _atlas.subscription_billing_schedule(subscription_id);
CREATE INDEX idx_sub_billing_date ON _atlas.subscription_billing_schedule(organization_id, billing_date) WHERE status = 'pending';

-- Subscription revenue schedule (ASC 606 / IFRS 15)
CREATE TABLE IF NOT EXISTS _atlas.subscription_revenue_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    subscription_id UUID NOT NULL REFERENCES _atlas.subscriptions(id),
    billing_schedule_id UUID REFERENCES _atlas.subscription_billing_schedule(id),
    period_name VARCHAR(20) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    revenue_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    deferred_amount NUMERIC(18, 2) NOT NULL DEFAULT 0,
    recognized_to_date NUMERIC(18, 2) NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'deferred', -- deferred, recognized, partially_recognized, reversed
    recognized_at TIMESTAMPTZ,
    journal_entry_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sub_revenue_subscription ON _atlas.subscription_revenue_schedule(subscription_id);
CREATE INDEX idx_sub_revenue_status ON _atlas.subscription_revenue_schedule(organization_id, status);
