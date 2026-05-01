-- ============================================================================
-- Loyalty Management (Oracle Fusion Cloud: CX > Loyalty Management)
-- ============================================================================
-- Manages loyalty programs, member enrollments, loyalty tiers, point
-- transactions (accrual, redemption, adjustment, expiration), reward
-- catalogs, loyalty promotions, and loyalty analytics dashboards.
-- ============================================================================

-- Loyalty Programs
CREATE TABLE IF NOT EXISTS _atlas.loyalty_programs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    program_type VARCHAR(30) NOT NULL DEFAULT 'points',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    currency_code VARCHAR(10) NOT NULL DEFAULT 'PTS',
    points_name VARCHAR(50) NOT NULL DEFAULT 'Points',
    enrollment_type VARCHAR(20) NOT NULL DEFAULT 'open',
    start_date DATE NOT NULL,
    end_date DATE,
    accrual_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    accrual_basis VARCHAR(20) NOT NULL DEFAULT 'amount',
    minimum_accrual_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    rounding_method VARCHAR(20) NOT NULL DEFAULT 'round',
    points_expiry_days INT,
    tier_qualification_period VARCHAR(20) NOT NULL DEFAULT 'yearly',
    auto_upgrade BOOLEAN NOT NULL DEFAULT true,
    auto_downgrade BOOLEAN NOT NULL DEFAULT false,
    max_points_per_member DOUBLE PRECISION,
    allow_point_transfer BOOLEAN NOT NULL DEFAULT false,
    allow_redemption BOOLEAN NOT NULL DEFAULT true,
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, program_number)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_programs_org ON _atlas.loyalty_programs(organization_id);
CREATE INDEX IF NOT EXISTS idx_loyalty_programs_status ON _atlas.loyalty_programs(status);

-- Loyalty Tier Definitions
CREATE TABLE IF NOT EXISTS _atlas.loyalty_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.loyalty_programs(id),
    tier_code VARCHAR(50) NOT NULL,
    tier_name VARCHAR(200) NOT NULL,
    tier_level INT NOT NULL DEFAULT 0,
    minimum_points DOUBLE PRECISION NOT NULL DEFAULT 0,
    maximum_points DOUBLE PRECISION,
    accrual_bonus_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
    benefits TEXT NOT NULL DEFAULT '',
    color VARCHAR(20) NOT NULL DEFAULT '',
    icon VARCHAR(200) NOT NULL DEFAULT '',
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(program_id, tier_code)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_tiers_program ON _atlas.loyalty_tiers(program_id);

-- Loyalty Members
CREATE TABLE IF NOT EXISTS _atlas.loyalty_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.loyalty_programs(id),
    member_number VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(200) NOT NULL DEFAULT '',
    customer_email VARCHAR(200) NOT NULL DEFAULT '',
    tier_id UUID REFERENCES _atlas.loyalty_tiers(id),
    tier_code VARCHAR(50) NOT NULL DEFAULT '',
    current_points DOUBLE PRECISION NOT NULL DEFAULT 0,
    lifetime_points DOUBLE PRECISION NOT NULL DEFAULT 0,
    redeemed_points DOUBLE PRECISION NOT NULL DEFAULT 0,
    expired_points DOUBLE PRECISION NOT NULL DEFAULT 0,
    enrollment_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    last_activity_date DATE,
    next_tier_points_remaining DOUBLE PRECISION,
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, member_number)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_members_org ON _atlas.loyalty_members(organization_id);
CREATE INDEX IF NOT EXISTS idx_loyalty_members_program ON _atlas.loyalty_members(program_id);
CREATE INDEX IF NOT EXISTS idx_loyalty_members_status ON _atlas.loyalty_members(status);
CREATE INDEX IF NOT EXISTS idx_loyalty_members_customer ON _atlas.loyalty_members(customer_id);

-- Point Transactions
CREATE TABLE IF NOT EXISTS _atlas.loyalty_point_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.loyalty_programs(id),
    member_id UUID NOT NULL REFERENCES _atlas.loyalty_members(id),
    transaction_number VARCHAR(50) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    points DOUBLE PRECISION NOT NULL,
    source_type VARCHAR(30) NOT NULL DEFAULT 'manual',
    source_id UUID,
    source_number VARCHAR(50) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    reference_amount DOUBLE PRECISION,
    reference_currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    tier_bonus_applied DOUBLE PRECISION NOT NULL DEFAULT 0,
    promo_bonus_applied DOUBLE PRECISION NOT NULL DEFAULT 0,
    expiry_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'posted',
    reversal_reason TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, transaction_number)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_ptxn_org ON _atlas.loyalty_point_transactions(organization_id);
CREATE INDEX IF NOT EXISTS idx_loyalty_ptxn_member ON _atlas.loyalty_point_transactions(member_id);
CREATE INDEX IF NOT EXISTS idx_loyalty_ptxn_type ON _atlas.loyalty_point_transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_loyalty_ptxn_status ON _atlas.loyalty_point_transactions(status);
CREATE INDEX IF NOT EXISTS idx_loyalty_ptxn_expiry ON _atlas.loyalty_point_transactions(expiry_date);

-- Rewards Catalog
CREATE TABLE IF NOT EXISTS _atlas.loyalty_rewards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.loyalty_programs(id),
    reward_code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    reward_type VARCHAR(30) NOT NULL DEFAULT 'merchandise',
    points_required DOUBLE PRECISION NOT NULL,
    cash_value DOUBLE PRECISION NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    tier_restriction VARCHAR(50) NOT NULL DEFAULT '',
    quantity_available INT,
    quantity_claimed INT NOT NULL DEFAULT 0,
    max_per_member INT,
    image_url VARCHAR(500) NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT true,
    start_date DATE,
    end_date DATE,
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, reward_code)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_rewards_program ON _atlas.loyalty_rewards(program_id);

-- Redemptions (reward claims by members)
CREATE TABLE IF NOT EXISTS _atlas.loyalty_redemptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    program_id UUID NOT NULL REFERENCES _atlas.loyalty_programs(id),
    member_id UUID NOT NULL REFERENCES _atlas.loyalty_members(id),
    reward_id UUID NOT NULL REFERENCES _atlas.loyalty_rewards(id),
    redemption_number VARCHAR(50) NOT NULL,
    points_spent DOUBLE PRECISION NOT NULL,
    quantity INT NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    fulfilled_at TIMESTAMPTZ,
    cancelled_reason TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, redemption_number)
);

CREATE INDEX IF NOT EXISTS idx_loyalty_redemptions_member ON _atlas.loyalty_redemptions(member_id);
CREATE INDEX IF NOT EXISTS idx_loyalty_redemptions_status ON _atlas.loyalty_redemptions(status);
