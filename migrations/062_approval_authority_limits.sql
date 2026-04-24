-- ═══════════════════════════════════════════════════════════════════════════════
-- Approval Authority Limits
-- Oracle Fusion Cloud BPM: Document Approval Limits / Signing Limits
-- ═══════════════════════════════════════════════════════════════════════════════
--
-- Oracle Fusion equivalent: BPM > Task Configuration > Document Approval Limits
--
-- Controls which transactions a user or role can approve based on
-- dollar amount thresholds, document types, and organizational scope
-- (business unit / cost center).
--
-- Supports:
--  - Per-user and per-role approval amount limits
--  - Document type scoping (PO, expense report, invoice, etc.)
--  - Business unit and cost center scoping
--  - Effective dating for temporary limit changes
--  - Full audit trail of every authority check

-- Approval authority limits
CREATE TABLE IF NOT EXISTS _atlas.approval_authority_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    limit_code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- Who this limit applies to
    owner_type VARCHAR(20) NOT NULL,           -- 'user' or 'role'
    user_id UUID,                               -- set when owner_type = 'user'
    role_name VARCHAR(200),                     -- set when owner_type = 'role'

    -- What this limit covers
    document_type VARCHAR(100) NOT NULL,        -- e.g. 'purchase_order', 'expense_report'
    approval_limit_amount TEXT NOT NULL,         -- maximum approval amount
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',

    -- Organizational scope
    business_unit_id UUID,
    cost_center VARCHAR(100),

    -- Lifecycle
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    effective_from DATE,
    effective_to DATE,

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, limit_code)
);

-- Audit trail for authority limit checks
CREATE TABLE IF NOT EXISTS _atlas.approval_authority_check_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    limit_id UUID,                              -- the limit that matched (if any)
    checked_user_id UUID NOT NULL,
    checked_role VARCHAR(200),
    document_type VARCHAR(100) NOT NULL,
    document_id UUID,
    requested_amount TEXT NOT NULL,
    applicable_limit TEXT NOT NULL,              -- the limit amount found
    result VARCHAR(20) NOT NULL,                 -- 'approved' or 'denied'
    reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_approval_authority_limits_org
    ON _atlas.approval_authority_limits(organization_id);
CREATE INDEX IF NOT EXISTS idx_approval_authority_limits_owner
    ON _atlas.approval_authority_limits(owner_type, user_id, role_name);
CREATE INDEX IF NOT EXISTS idx_approval_authority_limits_doc_type
    ON _atlas.approval_authority_limits(document_type);
CREATE INDEX IF NOT EXISTS idx_approval_authority_limits_status
    ON _atlas.approval_authority_limits(status);
CREATE INDEX IF NOT EXISTS idx_approval_authority_check_audit_org
    ON _atlas.approval_authority_check_audit(organization_id);
CREATE INDEX IF NOT EXISTS idx_approval_authority_check_audit_user
    ON _atlas.approval_authority_check_audit(checked_user_id);
CREATE INDEX IF NOT EXISTS idx_approval_authority_check_audit_result
    ON _atlas.approval_authority_check_audit(result);
