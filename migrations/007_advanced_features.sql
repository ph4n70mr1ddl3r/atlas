-- Atlas ERP - Advanced Oracle Fusion-inspired features
-- Structured filtering, bulk operations, comments, favorites, effective dating

-- ============================================================================
-- Comments / Notes on Records
-- Oracle Fusion: Conversation threads on business objects
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    parent_id UUID REFERENCES _atlas.comments(id) ON DELETE CASCADE,
    -- Author
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    user_name VARCHAR(255),
    -- Content
    body TEXT NOT NULL,
    body_format VARCHAR(20) DEFAULT 'plain', -- 'plain', 'markdown', 'rich'
    -- Mentions
    mentions JSONB DEFAULT '[]'::jsonb, -- list of user_ids mentioned
    -- Attachments
    attachments JSONB DEFAULT '[]'::jsonb, -- [{name, size, content_type, url}]
    -- Threading
    thread_root_id UUID REFERENCES _atlas.comments(id),
    depth INT DEFAULT 0,
    -- Reactions / flags
    is_pinned BOOLEAN DEFAULT false,
    is_internal BOOLEAN DEFAULT false, -- internal notes not visible to external users
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_comments_entity ON _atlas.comments(entity_type, entity_id, created_at);
CREATE INDEX idx_comments_parent ON _atlas.comments(parent_id);
CREATE INDEX idx_comments_thread ON _atlas.comments(thread_root_id) WHERE thread_root_id IS NOT NULL;
CREATE INDEX idx_comments_user ON _atlas.comments(user_id);
CREATE INDEX idx_comments_pinned ON _atlas.comments(entity_type, entity_id, is_pinned) WHERE is_pinned = true AND deleted_at IS NULL;

-- ============================================================================
-- Favorites / Bookmarks
-- Oracle Fusion: Users can favorite/bookmark records for quick access
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    -- Optional label/note
    label VARCHAR(255),
    notes TEXT,
    -- Display order
    display_order INT DEFAULT 0,
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    
    UNIQUE(user_id, entity_type, entity_id)
);

CREATE INDEX idx_favorites_user ON _atlas.favorites(user_id, entity_type);
CREATE INDEX idx_favorites_entity ON _atlas.favorites(entity_type, entity_id);

-- ============================================================================
-- Bulk Operation Jobs
-- Oracle Fusion: Mass update, mass delete, mass workflow action
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.bulk_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES _atlas.users(id),
    -- What to operate on
    entity_type VARCHAR(100) NOT NULL,
    operation VARCHAR(50) NOT NULL, -- 'update', 'delete', 'workflow_action'
    -- Filter to select records (same format as advanced filter)
    filter JSONB DEFAULT '{}',
    -- Or explicit list of record IDs
    record_ids JSONB DEFAULT '[]'::jsonb,
    -- Operation payload
    payload JSONB DEFAULT '{}', -- e.g., {"field": "status", "value": "active"} or {"action": "approve"}
    -- Progress
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    total_records INT DEFAULT 0,
    processed_records INT DEFAULT 0,
    succeeded_records INT DEFAULT 0,
    failed_records INT DEFAULT 0,
    -- Results
    errors JSONB DEFAULT '[]'::jsonb,
    -- Dry run mode (preview without executing)
    is_dry_run BOOLEAN DEFAULT false,
    -- Timestamps
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_bulk_ops_user ON _atlas.bulk_operations(user_id);
CREATE INDEX idx_bulk_ops_status ON _atlas.bulk_operations(status);

-- ============================================================================
-- Effective Dating
-- Oracle Fusion: Temporal data with effective date ranges
-- e.g., employee assignments, pricing, organizational structures
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.effective_dated_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    base_record_id UUID NOT NULL, -- The "current" or "latest" record this version belongs to
    -- Effective date range
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to DATE, -- NULL means "until further notice" / open-ended
    -- The full record data as of this effective period
    data JSONB NOT NULL DEFAULT '{}',
    -- Change reason
    change_reason VARCHAR(200),
    changed_by UUID REFERENCES _atlas.users(id),
    -- Tracking
    version INT DEFAULT 1,
    is_current BOOLEAN DEFAULT false, -- denormalized for quick lookup
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_eff_dated_entity ON _atlas.effective_dated_records(entity_type, base_record_id, effective_from);
CREATE INDEX idx_eff_dated_current ON _atlas.effective_dated_records(entity_type, base_record_id, is_current) WHERE is_current = true;
CREATE INDEX idx_eff_dated_date ON _atlas.effective_dated_records(effective_from, effective_to);

-- ============================================================================
-- File Attachments
-- Oracle Fusion: Attachment management with metadata
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Owner entity
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    -- File metadata
    file_name VARCHAR(500) NOT NULL,
    content_type VARCHAR(200) NOT NULL DEFAULT 'application/octet-stream',
    file_size BIGINT NOT NULL DEFAULT 0,
    -- Storage reference (filesystem path, S3 key, etc.)
    storage_path TEXT NOT NULL,
    storage_backend VARCHAR(20) DEFAULT 'local', -- 'local', 's3', 'gcs'
    -- Upload info
    uploaded_by UUID REFERENCES _atlas.users(id),
    -- Classification
    category VARCHAR(100), -- e.g., 'contract', 'invoice', 'resume'
    description TEXT,
    -- Versioning
    version INT DEFAULT 1,
    previous_version_id UUID REFERENCES _atlas.attachments(id),
    -- Access
    is_public BOOLEAN DEFAULT false,
    -- Meta
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_attachments_entity ON _atlas.attachments(entity_type, entity_id);
CREATE INDEX idx_attachments_uploaded_by ON _atlas.attachments(uploaded_by);
CREATE INDEX idx_attachments_category ON _atlas.attachments(category);
