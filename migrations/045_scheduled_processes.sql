-- Scheduled Processes (Oracle Fusion Enterprise Scheduler Service)
-- Oracle Fusion: Navigator > Tools > Scheduled Processes
--
-- Defines process templates, submitted process instances, parameters,
-- execution logs, and schedule definitions for recurring jobs.

-- Process templates: reusable definitions for scheduled jobs
CREATE TABLE IF NOT EXISTS _atlas.scheduled_process_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    process_type VARCHAR(50) NOT NULL DEFAULT 'report',  -- report, import, export, batch, custom
    executor_type VARCHAR(50) NOT NULL DEFAULT 'built_in', -- built_in, external, plugin
    executor_config JSONB DEFAULT '{}',
    parameters JSONB DEFAULT '[]',
    default_parameters JSONB DEFAULT '{}',
    timeout_minutes INT DEFAULT 60,
    max_retries INT DEFAULT 0,
    retry_delay_minutes INT DEFAULT 5,
    requires_approval BOOLEAN DEFAULT false,
    approval_chain_id UUID,
    is_active BOOLEAN DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Submitted scheduled process instances
CREATE TABLE IF NOT EXISTS _atlas.scheduled_processes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    template_id UUID REFERENCES _atlas.scheduled_process_templates(id),
    template_code VARCHAR(100),
    process_name VARCHAR(200) NOT NULL,
    process_type VARCHAR(50) NOT NULL DEFAULT 'report',
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    -- pending, scheduled, running, completed, failed, cancelled, waiting_for_approval
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    -- low, normal, high, urgent
    submitted_by UUID NOT NULL,
    submitted_at TIMESTAMPTZ DEFAULT now(),
    scheduled_start_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancelled_by UUID,
    cancel_reason TEXT,
    last_heartbeat_at TIMESTAMPTZ,
    retry_count INT DEFAULT 0,
    max_retries INT DEFAULT 0,
    timeout_minutes INT DEFAULT 60,
    progress_percent INT DEFAULT 0,
    parameters JSONB DEFAULT '{}',
    result_summary TEXT,
    output_file_url TEXT,
    output_format VARCHAR(20) DEFAULT 'json',
    log_output TEXT,
    error_message TEXT,
    parent_process_id UUID,  -- for sub-processes
    recurrence_id UUID,      -- links to recurrence schedule if spawned by one
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Recurrence schedules for recurring process submissions
CREATE TABLE IF NOT EXISTS _atlas.scheduled_process_recurrences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    template_id UUID REFERENCES _atlas.scheduled_process_templates(id) NOT NULL,
    template_code VARCHAR(100),
    parameters JSONB DEFAULT '{}',
    recurrence_type VARCHAR(30) NOT NULL DEFAULT 'daily',
    -- daily, weekly, monthly, cron
    recurrence_config JSONB DEFAULT '{}',
    -- { "days_of_week": [...], "day_of_month": 1, "cron_expression": "...", "time": "HH:MM" }
    start_date DATE NOT NULL,
    end_date DATE,
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    run_count INT DEFAULT 0,
    max_runs INT,
    is_active BOOLEAN DEFAULT true,
    submitted_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Process execution log entries (detailed step-by-step log)
CREATE TABLE IF NOT EXISTS _atlas.scheduled_process_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    process_id UUID REFERENCES _atlas.scheduled_processes(id) NOT NULL,
    log_level VARCHAR(20) NOT NULL DEFAULT 'info',
    -- debug, info, warn, error
    message TEXT NOT NULL,
    details JSONB,
    step_name VARCHAR(200),
    duration_ms INT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_sp_templates_org ON _atlas.scheduled_process_templates(organization_id);
CREATE INDEX IF NOT EXISTS idx_sp_templates_active ON _atlas.scheduled_process_templates(organization_id, is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_sp_org_status ON _atlas.scheduled_processes(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_sp_submitted_by ON _atlas.scheduled_processes(submitted_by);
CREATE INDEX IF NOT EXISTS idx_sp_scheduled_start ON _atlas.scheduled_processes(scheduled_start_at) WHERE status = 'scheduled';
CREATE INDEX IF NOT EXISTS idx_sp_template ON _atlas.scheduled_processes(template_id);
CREATE INDEX IF NOT EXISTS idx_spr_template ON _atlas.scheduled_process_recurrences(template_id);
CREATE INDEX IF NOT EXISTS idx_spr_next_run ON _atlas.scheduled_process_recurrences(next_run_at) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_spl_process ON _atlas.scheduled_process_logs(process_id);
