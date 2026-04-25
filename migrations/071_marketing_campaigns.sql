-- Marketing Campaign Management
-- Oracle Fusion: CX Marketing > Campaigns
-- Provides campaign types, campaigns, campaign members, responses, and ROI analytics.

CREATE TABLE IF NOT EXISTS _atlas.campaign_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    channel VARCHAR(50) NOT NULL DEFAULT 'email',
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE TABLE IF NOT EXISTS _atlas.marketing_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    campaign_number VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    campaign_type_id UUID REFERENCES _atlas.campaign_types(id),
    campaign_type_name VARCHAR(200),
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    channel VARCHAR(50) NOT NULL DEFAULT 'email',
    budget DOUBLE PRECISION DEFAULT 0,
    actual_cost DOUBLE PRECISION DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    start_date DATE,
    end_date DATE,
    owner_id UUID,
    owner_name VARCHAR(200),
    expected_responses INTEGER DEFAULT 0,
    expected_revenue DOUBLE PRECISION DEFAULT 0,
    actual_responses INTEGER DEFAULT 0,
    actual_revenue DOUBLE PRECISION DEFAULT 0,
    converted_leads INTEGER DEFAULT 0,
    converted_opportunities INTEGER DEFAULT 0,
    converted_won INTEGER DEFAULT 0,
    parent_campaign_id UUID,
    parent_campaign_name VARCHAR(500),
    tags JSONB DEFAULT '[]',
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    activated_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, campaign_number)
);

CREATE TABLE IF NOT EXISTS _atlas.campaign_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    campaign_id UUID NOT NULL REFERENCES _atlas.marketing_campaigns(id) ON DELETE CASCADE,
    contact_id UUID,
    contact_name VARCHAR(500),
    contact_email VARCHAR(500),
    lead_id UUID,
    lead_number VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'invited',
    response VARCHAR(50),
    responded_at TIMESTAMPTZ,
    converted_contact_id UUID,
    converted_lead_id UUID,
    converted_opportunity_id UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS _atlas.campaign_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    campaign_id UUID NOT NULL REFERENCES _atlas.marketing_campaigns(id) ON DELETE CASCADE,
    member_id UUID REFERENCES _atlas.campaign_members(id) ON DELETE CASCADE,
    response_type VARCHAR(50) NOT NULL,
    contact_id UUID,
    contact_name VARCHAR(500),
    contact_email VARCHAR(500),
    lead_id UUID,
    description TEXT,
    value DOUBLE PRECISION DEFAULT 0,
    currency_code VARCHAR(3) DEFAULT 'USD',
    source_url TEXT,
    responded_at TIMESTAMPTZ DEFAULT now(),
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_campaign_types_org ON _atlas.campaign_types(organization_id);
CREATE INDEX IF NOT EXISTS idx_marketing_campaigns_org ON _atlas.marketing_campaigns(organization_id);
CREATE INDEX IF NOT EXISTS idx_marketing_campaigns_status ON _atlas.marketing_campaigns(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_campaign_members_campaign ON _atlas.campaign_members(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_members_contact ON _atlas.campaign_members(contact_id);
CREATE INDEX IF NOT EXISTS idx_campaign_responses_campaign ON _atlas.campaign_responses(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_responses_member ON _atlas.campaign_responses(member_id);
