-- ═══════════════════════════════════════════════════════════════════════════════
-- Data Archiving and Retention Management
-- Oracle Fusion Cloud: Information Lifecycle Management (ILM)
-- ═══════════════════════════════════════════════════════════════════════════════
--
-- Oracle Fusion equivalent: Tools > Information Lifecycle Management
--
-- Provides configurable retention policies, data archival, purge
-- management, legal holds, and restore capabilities for compliance
-- and data lifecycle governance.
--
-- Key capabilities:
--  - Retention policies per entity type with configurable retention periods
--  - Archive policies (how and where data is archived)
--  - Purge policies (when data can be permanently deleted)
--  - Legal holds to prevent archival/purge of records under litigation
--  - Restore archived records back to active tables
--  - Full audit trail of every archive/purge/restore operation

-- Retention policies define how long data of a given entity type
-- must be retained before it can be archived or purged.
CREATE TABLE IF NOT EXISTS _atlas.retention_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    policy_code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,

    -- What entity type this policy applies to
    entity_type VARCHAR(200) NOT NULL,

    -- Retention period
    retention_days INT NOT NULL DEFAULT 365,
    -- "archive", "purge", "archive_then_purge"
    action_type VARCHAR(50) NOT NULL DEFAULT 'archive_then_purge',
    -- Additional days after archive before purge (when action_type = archive_then_purge)
    purge_after_days INT,

    -- Condition expression for filtering which records qualify
    condition_expression TEXT,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'active',

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, policy_code)
);

-- Legal holds prevent archival or purging of specific records
-- regardless of retention policy settings.
CREATE TABLE IF NOT EXISTS _atlas.legal_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    hold_number VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- "active", "released", "expired"
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    -- Reason for the hold (e.g., litigation case number)
    reason TEXT,
    -- Legal case or reference identifier
    case_reference VARCHAR(200),
    -- Who authorized the hold
    authorized_by UUID,
    -- When the hold was released (if released)
    released_at TIMESTAMPTZ,
    released_by UUID,
    release_reason TEXT,
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to DATE,

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, hold_number)
);

-- Legal hold items: which specific records are under hold
CREATE TABLE IF NOT EXISTS _atlas.legal_hold_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    legal_hold_id UUID NOT NULL REFERENCES _atlas.legal_holds(id) ON DELETE CASCADE,
    -- The entity type and record ID under hold
    entity_type VARCHAR(200) NOT NULL,
    record_id UUID NOT NULL,

    created_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(legal_hold_id, entity_type, record_id)
);

-- Archived records: tracks records that have been archived
CREATE TABLE IF NOT EXISTS _atlas.archived_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    -- Source entity info
    entity_type VARCHAR(200) NOT NULL,
    original_record_id UUID NOT NULL,

    -- Snapshot of the original data
    original_data JSONB NOT NULL,

    -- Which policy triggered the archival
    retention_policy_id UUID,
    -- Archive batch (for grouping archives)
    archive_batch_id UUID,

    -- Status: "archived", "restored", "purged"
    status VARCHAR(20) NOT NULL DEFAULT 'archived',

    -- Timestamps from original record
    original_created_at TIMESTAMPTZ,
    original_updated_at TIMESTAMPTZ,

    -- When it was archived
    archived_at TIMESTAMPTZ DEFAULT now(),
    archived_by UUID,

    -- Restoration tracking
    restored_at TIMESTAMPTZ,
    restored_by UUID,

    -- Purge tracking
    purged_at TIMESTAMPTZ,
    purged_by UUID,

    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Archive batches: groups of records archived together
CREATE TABLE IF NOT EXISTS _atlas.archive_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    batch_number VARCHAR(100) NOT NULL,

    -- Which policy this batch was created under
    retention_policy_id UUID,
    entity_type VARCHAR(200) NOT NULL,

    -- Status: "pending", "in_progress", "completed", "failed", "reversed"
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    total_records INT NOT NULL DEFAULT 0,
    archived_records INT NOT NULL DEFAULT 0,
    failed_records INT NOT NULL DEFAULT 0,

    -- Criteria used for this batch
    criteria JSONB DEFAULT '{}',

    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    UNIQUE(organization_id, batch_number)
);

-- Archive audit trail
CREATE TABLE IF NOT EXISTS _atlas.archive_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,

    -- Operation details
    operation VARCHAR(50) NOT NULL,   -- "archive", "restore", "purge", "legal_hold", "legal_hold_release"
    entity_type VARCHAR(200) NOT NULL,
    record_id UUID,

    -- Reference to batch or legal hold
    batch_id UUID,
    legal_hold_id UUID,
    retention_policy_id UUID,

    -- Result
    result VARCHAR(20) NOT NULL,      -- "success", "failed", "skipped"
    details TEXT,

    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),

    metadata JSONB DEFAULT '{}'
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_retention_policies_org ON _atlas.retention_policies(organization_id);
CREATE INDEX IF NOT EXISTS idx_retention_policies_entity ON _atlas.retention_policies(entity_type);
CREATE INDEX IF NOT EXISTS idx_retention_policies_status ON _atlas.retention_policies(status);

CREATE INDEX IF NOT EXISTS idx_legal_holds_org ON _atlas.legal_holds(organization_id);
CREATE INDEX IF NOT EXISTS idx_legal_holds_status ON _atlas.legal_holds(status);

CREATE INDEX IF NOT EXISTS idx_legal_hold_items_hold ON _atlas.legal_hold_items(legal_hold_id);
CREATE INDEX IF NOT EXISTS idx_legal_hold_items_record ON _atlas.legal_hold_items(entity_type, record_id);

CREATE INDEX IF NOT EXISTS idx_archived_records_org ON _atlas.archived_records(organization_id);
CREATE INDEX IF NOT EXISTS idx_archived_records_entity ON _atlas.archived_records(entity_type);
CREATE INDEX IF NOT EXISTS idx_archived_records_original ON _atlas.archived_records(original_record_id);
CREATE INDEX IF NOT EXISTS idx_archived_records_status ON _atlas.archived_records(status);
CREATE INDEX IF NOT EXISTS idx_archived_records_batch ON _atlas.archived_records(archive_batch_id);

CREATE INDEX IF NOT EXISTS idx_archive_batches_org ON _atlas.archive_batches(organization_id);
CREATE INDEX IF NOT EXISTS idx_archive_batches_status ON _atlas.archive_batches(status);

CREATE INDEX IF NOT EXISTS idx_archive_audit_org ON _atlas.archive_audit(organization_id);
CREATE INDEX IF NOT EXISTS idx_archive_audit_operation ON _atlas.archive_audit(operation);
CREATE INDEX IF NOT EXISTS idx_archive_audit_performed_at ON _atlas.archive_audit(performed_at);
