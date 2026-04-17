-- Atlas ERP - Notifications, Saved Searches, Approval Chains, and Duplicate Detection
-- Oracle Fusion-inspired enhancements

-- ============================================================================
-- Notification System
-- Oracle Fusion: Bell icon notifications for workflow actions, approvals, escalations
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    notification_type VARCHAR(50) NOT NULL, -- 'workflow_action', 'approval_required', 'escalation', 'system', 'mention', 'assignment'
    priority VARCHAR(20) DEFAULT 'normal', -- 'low', 'normal', 'high', 'urgent'
    title VARCHAR(500) NOT NULL,
    message TEXT,
    entity_type VARCHAR(100),
    entity_id UUID,
    action_url VARCHAR(1000),
    -- Workflow action info
    workflow_name VARCHAR(100),
    from_state VARCHAR(100),
    to_state VARCHAR(100),
    action VARCHAR(100),
    performed_by UUID,
    -- Read status
    is_read BOOLEAN DEFAULT false,
    read_at TIMESTAMPTZ,
    -- Dismissal
    is_dismissed BOOLEAN DEFAULT false,
    dismissed_at TIMESTAMPTZ,
    -- Scheduling
    scheduled_for TIMESTAMPTZ, -- For delayed notifications / escalation timers
    sent_at TIMESTAMPTZ,
    -- Delivery channels
    channels JSONB DEFAULT '["in_app"]'::jsonb, -- ["in_app", "email", "sms", "webhook"]
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_notifications_user ON _atlas.notifications(user_id, is_read, is_dismissed);
CREATE INDEX idx_notifications_org ON _atlas.notifications(organization_id);
CREATE INDEX idx_notifications_type ON _atlas.notifications(notification_type);
CREATE INDEX idx_notifications_entity ON _atlas.notifications(entity_type, entity_id);
CREATE INDEX idx_notifications_scheduled ON _atlas.notifications(scheduled_for) WHERE scheduled_for IS NOT NULL AND sent_at IS NULL;
CREATE INDEX idx_notifications_created ON _atlas.notifications(created_at DESC);

-- ============================================================================
-- Notification Preferences (per user)
-- Oracle Fusion: Users can configure which notifications they receive and how
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    organization_id UUID NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    enabled BOOLEAN DEFAULT true,
    channels JSONB DEFAULT '["in_app"]'::jsonb,
    quiet_hours_start TIME, -- e.g., '22:00'
    quiet_hours_end TIME,     -- e.g., '08:00'
    digest_frequency VARCHAR(20) DEFAULT 'immediate', -- 'immediate', 'hourly', 'daily', 'weekly'
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(user_id, notification_type)
);

-- ============================================================================
-- Saved Searches / Personalized Views
-- Oracle Fusion: Users can save common filter combinations and list configurations
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.saved_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    name VARCHAR(200) NOT NULL,
    description TEXT,
    entity_type VARCHAR(100) NOT NULL,
    -- Filter configuration
    filters JSONB DEFAULT '[]'::jsonb,
    sort_by VARCHAR(100) DEFAULT 'created_at',
    sort_direction VARCHAR(10) DEFAULT 'desc',
    -- Column visibility & order
    columns JSONB DEFAULT '[]'::jsonb, -- list of field names in display order
    columns_widths JSONB DEFAULT '{}'::jsonb, -- field_name -> width_px
    -- Pagination
    page_size INT DEFAULT 20,
    -- Sharing
    is_shared BOOLEAN DEFAULT false,
    is_default BOOLEAN DEFAULT false,
    -- Meta
    color VARCHAR(20),
    icon VARCHAR(50),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_saved_searches_user ON _atlas.saved_searches(user_id, entity_type);
CREATE INDEX idx_saved_searches_entity ON _atlas.saved_searches(entity_type);
CREATE INDEX idx_saved_searches_shared ON _atlas.saved_searches(organization_id, is_shared) WHERE is_shared = true;

-- ============================================================================
-- Approval Chains (Multi-Level Approval)
-- Oracle Fusion: FAC-supported multi-level approval hierarchies with delegation
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.approval_chains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    entity_type VARCHAR(100) NOT NULL, -- which entity type this applies to
    -- Condition for when this chain activates (expression)
    condition_expression TEXT, -- e.g., "total_amount > 10000" 
    -- Chain definition as ordered list of approval levels
    chain_definition JSONB NOT NULL DEFAULT '[]'::jsonb, 
    -- [{"level": 1, "type": "role", "roles": ["manager"], "auto_approve_after_hours": 48}]    
    -- Escalation rules
    escalation_enabled BOOLEAN DEFAULT true,
    escalation_hours INT DEFAULT 48,
    escalation_to_roles JSONB DEFAULT '[]'::jsonb,
    -- Delegation
    allow_delegation BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_approval_chains_entity ON _atlas.approval_chains(entity_type);

-- ============================================================================
-- Approval Requests (runtime approval instances)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.approval_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    chain_id UUID REFERENCES _atlas.approval_chains(id),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    -- Current approval state
    current_level INT DEFAULT 1,
    total_levels INT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'escalated', 'cancelled', 'timed_out'
    -- Requester
    requested_by UUID NOT NULL REFERENCES _atlas.users(id),
    requested_at TIMESTAMPTZ DEFAULT now(),
    -- Completion
    completed_at TIMESTAMPTZ,
    completed_by UUID,
    -- Escalation tracking
    last_escalated_at TIMESTAMPTZ,
    escalation_count INT DEFAULT 0,
    -- Meta
    title VARCHAR(500),
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_approval_requests_entity ON _atlas.approval_requests(entity_type, entity_id);
CREATE INDEX idx_approval_requests_status ON _atlas.approval_requests(status);
CREATE INDEX idx_approval_requests_requested_by ON _atlas.approval_requests(requested_by);

-- ============================================================================
-- Approval Steps (individual steps within a chain)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.approval_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    approval_request_id UUID NOT NULL REFERENCES _atlas.approval_requests(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    level INT NOT NULL,
    -- Approver
    approver_type VARCHAR(20) NOT NULL, -- 'role', 'user', 'auto'
    approver_role VARCHAR(100),
    approver_user_id UUID REFERENCES _atlas.users(id),
    -- Delegation
    is_delegated BOOLEAN DEFAULT false,
    delegated_by UUID REFERENCES _atlas.users(id),
    delegated_to UUID REFERENCES _atlas.users(id),
    -- Step result
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'delegated', 'escalated', 'timed_out'
    action_at TIMESTAMPTZ,
    action_by UUID REFERENCES _atlas.users(id),
    comment TEXT,
    -- Auto-approve
    auto_approve_after_hours INT,
    -- Meta
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_approval_steps_request ON _atlas.approval_steps(approval_request_id);
CREATE INDEX idx_approval_steps_approver ON _atlas.approval_steps(approver_user_id) WHERE approver_user_id IS NOT NULL;
CREATE INDEX idx_approval_steps_pending ON _atlas.approval_steps(status) WHERE status = 'pending';

-- ============================================================================
-- Duplicate Detection Rules
-- Oracle Fusion: Prevent creating duplicate records based on matching criteria
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.duplicate_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    description TEXT,
    -- Match criteria: which fields to compare and how
    -- [{"field": "email", "match_type": "exact"}, {"field": "name", "match_type": "fuzzy", "threshold": 0.8}]
    match_criteria JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Filter: only check duplicates within records matching this condition
    filter_condition JSONB DEFAULT '{}'::jsonb,
    -- Action when duplicate found: 'block', 'warn', 'merge_suggest'
    on_duplicate VARCHAR(20) DEFAULT 'warn',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, entity_type, name)
);

CREATE INDEX idx_duplicate_rules_entity ON _atlas.duplicate_rules(entity_type);

-- ============================================================================
-- Computed Field Cache
-- Stores computed field values to avoid recalculating on every read
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.computed_field_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    field_name VARCHAR(100) NOT NULL,
    computed_value JSONB,
    formula_version INT DEFAULT 1,
    computed_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(entity_type, entity_id, field_name)
);

CREATE INDEX idx_computed_cache_entity ON _atlas.computed_field_cache(entity_type, entity_id);

-- ============================================================================
-- Data Import Jobs
-- Oracle Fusion: Track import jobs with progress, validation results, and errors
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    entity_type VARCHAR(100) NOT NULL,
    format VARCHAR(20) NOT NULL DEFAULT 'csv', -- 'csv', 'json'
    -- Job status
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'validating', 'importing', 'completed', 'failed', 'cancelled'
    total_rows INT DEFAULT 0,
    processed_rows INT DEFAULT 0,
    imported_rows INT DEFAULT 0,
    failed_rows INT DEFAULT 0,
    skipped_rows INT DEFAULT 0,
    -- File info
    original_filename VARCHAR(500),
    file_size_bytes BIGINT,
    -- Field mapping (CSV column -> entity field)
    field_mapping JSONB DEFAULT '{}',
    -- Import options
    upsert_mode BOOLEAN DEFAULT false, -- update existing records
    skip_validation BOOLEAN DEFAULT false,
    stop_on_error BOOLEAN DEFAULT false,
    -- Results
    validation_errors JSONB DEFAULT '[]'::jsonb,
    import_errors JSONB DEFAULT '[]'::jsonb,
    -- Duplicate handling
    duplicate_action VARCHAR(20) DEFAULT 'skip', -- 'skip', 'overwrite', 'warn'
    duplicates_found INT DEFAULT 0,
    -- Timestamps
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_import_jobs_user ON _atlas.import_jobs(user_id);
CREATE INDEX idx_import_jobs_status ON _atlas.import_jobs(status);