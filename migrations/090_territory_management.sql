-- Territory Management (Oracle Fusion CX Sales > Territory Management)
-- Tables for sales territories, hierarchy, member assignments, routing rules, and quotas.

CREATE TABLE IF NOT EXISTS _atlas.territories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    territory_type VARCHAR(50) NOT NULL DEFAULT 'geography',
    parent_id UUID REFERENCES _atlas.territories(id),
    owner_id UUID,
    owner_name VARCHAR(200),
    is_active BOOLEAN DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX IF NOT EXISTS idx_territories_org ON _atlas.territories(organization_id);
CREATE INDEX IF NOT EXISTS idx_territories_parent ON _atlas.territories(parent_id);
CREATE INDEX IF NOT EXISTS idx_territories_type ON _atlas.territories(territory_type);
CREATE INDEX IF NOT EXISTS idx_territories_active ON _atlas.territories(is_active);

CREATE TABLE IF NOT EXISTS _atlas.territory_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    territory_id UUID NOT NULL REFERENCES _atlas.territories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    user_name VARCHAR(200) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    is_active BOOLEAN DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_territory_members_territory ON _atlas.territory_members(territory_id);
CREATE INDEX IF NOT EXISTS idx_territory_members_user ON _atlas.territory_members(user_id);
CREATE INDEX IF NOT EXISTS idx_territory_members_role ON _atlas.territory_members(role);

CREATE TABLE IF NOT EXISTS _atlas.territory_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    territory_id UUID NOT NULL REFERENCES _atlas.territories(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    field_name VARCHAR(100) NOT NULL,
    match_operator VARCHAR(50) NOT NULL,
    match_value TEXT NOT NULL DEFAULT '',
    priority INT NOT NULL DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_territory_rules_territory ON _atlas.territory_rules(territory_id);
CREATE INDEX IF NOT EXISTS idx_territory_rules_entity ON _atlas.territory_rules(entity_type);

CREATE TABLE IF NOT EXISTS _atlas.territory_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    territory_id UUID NOT NULL REFERENCES _atlas.territories(id) ON DELETE CASCADE,
    period_name VARCHAR(100) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    revenue_quota NUMERIC(18,2) NOT NULL DEFAULT 0,
    actual_revenue NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(territory_id, period_name)
);

CREATE INDEX IF NOT EXISTS idx_territory_quotas_territory ON _atlas.territory_quotas(territory_id);
CREATE INDEX IF NOT EXISTS idx_territory_quotas_period ON _atlas.territory_quotas(period_name);
