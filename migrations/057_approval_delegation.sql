-- Approval Delegation Rules (Oracle Fusion BPM Worklist > Rules > Configure Delegation)
-- Users can proactively set up delegation rules that say:
-- "While I'm away from X to Y date, delegate all my approvals to user Z"
-- This is distinct from per-step delegation and enables automatic routing.

-- Delegation Rules
CREATE TABLE IF NOT EXISTS _atlas.approval_delegation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    delegator_id UUID NOT NULL,           -- the user who is delegating their approvals
    delegate_to_id UUID NOT NULL,         -- the user who will receive the delegated approvals
    rule_name VARCHAR(200) NOT NULL,
    description TEXT,
    delegation_type VARCHAR(30) NOT NULL DEFAULT 'all',  -- 'all', 'by_category', 'by_role', 'by_entity'
    -- for type 'by_category': categories as JSON array
    categories JSONB NOT NULL DEFAULT '[]',
    -- for type 'by_role': roles as JSON array
    roles JSONB NOT NULL DEFAULT '[]',
    -- for type 'by_entity': entity types as JSON array
    entity_types JSONB NOT NULL DEFAULT '[]',
    start_date DATE NOT NULL,             -- when delegation becomes active
    end_date DATE NOT NULL,               -- when delegation expires
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- automatic activation/expiry
    auto_activate BOOLEAN NOT NULL DEFAULT true,
    auto_expire BOOLEAN NOT NULL DEFAULT true,
    -- status: 'scheduled', 'active', 'expired', 'cancelled'
    status VARCHAR(30) NOT NULL DEFAULT 'scheduled',
    activated_at TIMESTAMPTZ,
    expired_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancellation_reason TEXT,
    -- metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- prevent exact duplicate rules
    UNIQUE(organization_id, delegator_id, delegate_to_id, start_date, end_date, rule_name)
);

-- Delegation History (tracks when delegations were actually used)
CREATE TABLE IF NOT EXISTS _atlas.approval_delegation_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    delegation_rule_id UUID NOT NULL REFERENCES _atlas.approval_delegation_rules(id) ON DELETE CASCADE,
    original_approver_id UUID NOT NULL,
    delegated_to_id UUID NOT NULL,
    approval_step_id UUID,                -- the approval step that was delegated
    approval_request_id UUID,             -- the approval request
    entity_type VARCHAR(100),
    entity_id UUID,
    action_taken VARCHAR(30),             -- 'approved', 'rejected', 'pending'
    action_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_delegation_rules_org ON _atlas.approval_delegation_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_delegation_rules_delegator ON _atlas.approval_delegation_rules(organization_id, delegator_id);
CREATE INDEX IF NOT EXISTS idx_delegation_rules_delegate ON _atlas.approval_delegation_rules(organization_id, delegate_to_id);
CREATE INDEX IF NOT EXISTS idx_delegation_rules_status ON _atlas.approval_delegation_rules(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_delegation_rules_dates ON _atlas.approval_delegation_rules(start_date, end_date) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_delegation_history_rule ON _atlas.approval_delegation_history(delegation_rule_id);
CREATE INDEX IF NOT EXISTS idx_delegation_history_original ON _atlas.approval_delegation_history(original_approver_id);
CREATE INDEX IF NOT EXISTS idx_delegation_history_delegated ON _atlas.approval_delegation_history(delegated_to_id);
