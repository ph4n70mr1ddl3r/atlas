-- Atlas ERP Database Schema
-- Initial migration

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Atlas internal schema (system tables)
CREATE SCHEMA IF NOT EXISTS _atlas;

-- ============================================================================
-- Users and Authentication
-- ============================================================================

CREATE TABLE _atlas.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    roles JSONB DEFAULT '[]',
    organization_id UUID NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON _atlas.users(email);
CREATE INDEX idx_users_org ON _atlas.users(organization_id);

-- ============================================================================
-- Organizations (Tenants)
-- ============================================================================

CREATE TABLE _atlas.organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    parent_id UUID REFERENCES _atlas.organizations(id),
    settings JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_orgs_code ON _atlas.organizations(code);
CREATE INDEX idx_orgs_parent ON _atlas.organizations(parent_id);

-- Add foreign key to users
ALTER TABLE _atlas.users 
    ADD CONSTRAINT fk_users_org 
    FOREIGN KEY (organization_id) 
    REFERENCES _atlas.organizations(id);

-- ============================================================================
-- Entity Definitions
-- ============================================================================

CREATE TABLE _atlas.entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    label VARCHAR(200) NOT NULL,
    plural_label VARCHAR(200) NOT NULL,
    table_name VARCHAR(100),
    description TEXT,
    
    -- Field definitions as JSON
    fields JSONB NOT NULL DEFAULT '[]',
    indexes JSONB DEFAULT '[]',
    workflow JSONB,
    security JSONB,
    
    -- Settings
    is_audit_enabled BOOLEAN DEFAULT true,
    is_soft_delete BOOLEAN DEFAULT true,
    icon VARCHAR(50),
    color VARCHAR(20),
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_entities_name ON _atlas.entities(name);

-- ============================================================================
-- Configuration Versions (for hot-reload)
-- ============================================================================

CREATE TABLE _atlas.config_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_name VARCHAR(100),
    config_name VARCHAR(100),
    version BIGINT NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID REFERENCES _atlas.users(id)
);

CREATE INDEX idx_config_versions_entity ON _atlas.config_versions(entity_name);
CREATE INDEX idx_config_versions_name ON _atlas.config_versions(config_name);
CREATE INDEX idx_config_versions_version ON _atlas.config_versions(entity_name, version DESC);

-- ============================================================================
-- Audit Log
-- ============================================================================

CREATE TABLE _atlas.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL,
    old_data JSONB,
    new_data JSONB,
    changed_by UUID REFERENCES _atlas.users(id),
    changed_at TIMESTAMPTZ DEFAULT now(),
    session_id UUID,
    ip_address VARCHAR(45),
    user_agent TEXT
);

CREATE INDEX idx_audit_entity ON _atlas.audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_changed_by ON _atlas.audit_log(changed_by);
CREATE INDEX idx_audit_changed_at ON _atlas.audit_log(changed_at DESC);

-- Partition audit log by month for better performance (optional)
-- See: https://www.postgresql.org/docs/current/sql-createtable.html#SQL-CREATETABLE-PARTITIONING

-- ============================================================================
-- Workflow State Tracking
-- ============================================================================

CREATE TABLE _atlas.workflow_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    workflow_name VARCHAR(100) NOT NULL,
    current_state VARCHAR(100) NOT NULL,
    state_type VARCHAR(20) NOT NULL,
    history JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    
    UNIQUE(entity_type, record_id)
);

CREATE INDEX idx_workflow_record ON _atlas.workflow_states(entity_type, record_id);
CREATE INDEX idx_workflow_state ON _atlas.workflow_states(workflow_name, current_state);

-- ============================================================================
-- System Configuration
-- ============================================================================

CREATE TABLE _atlas.system_config (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    value_type VARCHAR(20) DEFAULT 'string',
    description TEXT,
    is_sensitive BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- ============================================================================
-- Default Admin User
-- ============================================================================

-- Insert default organization
INSERT INTO _atlas.organizations (id, name, code) VALUES 
    ('00000000-0000-0000-0000-000000000001', 'Default Organization', 'DEFAULT');

-- Insert admin user (password: admin123)
-- Password hash generated with Argon2id
INSERT INTO _atlas.users (id, email, name, password_hash, roles, organization_id) VALUES 
    ('00000000-0000-0000-0000-000000000002', 'admin@atlas.local', 'System Administrator', 
     '$argon2id$v=19$m=19456,t=2,p=1$d/ce2R9A0BCBBqiaYeGHUw$iGegymLltUV9IKxr7cixQqWUvamhHdjKhjEcH7qcGmI', 
     '[["admin", "system"]] '::jsonb,
     '00000000-0000-0000-0000-000000000001');

-- ============================================================================
-- System Configuration Defaults
-- ============================================================================

INSERT INTO _atlas.system_config (key, value, value_type, description) VALUES
    ('app.name', '"Atlas ERP"', 'string', 'Application name'),
    ('app.version', '"0.1.0"', 'string', 'Application version'),
    ('app.timezone', '"UTC"', 'string', 'Default timezone'),
    ('session.timeout_minutes', '60', 'number', 'Session timeout in minutes'),
    ('security.jwt_expiry_hours', '24', 'number', 'JWT token expiry'),
    ('ui.items_per_page', '20', 'number', 'Default pagination size');

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON SCHEMA _atlas IS 'Atlas ERP internal schema for system tables';
COMMENT ON TABLE _atlas.entities IS 'Dynamic entity definitions stored declaratively';
COMMENT ON TABLE _atlas.audit_log IS 'Complete audit trail for all data changes';
COMMENT ON TABLE _atlas.workflow_states IS 'Runtime workflow state for records';
