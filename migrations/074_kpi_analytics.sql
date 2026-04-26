-- KPI & Embedded Analytics (Oracle Fusion OTBI-inspired)
-- Provides KPI definitions, data point tracking, dashboards, and widgets

-- KPI Definitions
CREATE TABLE IF NOT EXISTS _atlas.kpi_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL DEFAULT 'general',
    unit_of_measure VARCHAR(50) NOT NULL DEFAULT 'number',
    direction VARCHAR(30) NOT NULL DEFAULT 'higher_is_better',
    target_value TEXT NOT NULL DEFAULT '0',
    warning_threshold TEXT,
    critical_threshold TEXT,
    data_source_query TEXT,
    evaluation_frequency VARCHAR(30) NOT NULL DEFAULT 'manual',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- KPI Data Points (time-series)
CREATE TABLE IF NOT EXISTS _atlas.kpi_data_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    kpi_id UUID NOT NULL REFERENCES _atlas.kpi_definitions(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    period_start DATE,
    period_end DATE,
    status VARCHAR(30) NOT NULL DEFAULT 'no_target',
    notes TEXT,
    recorded_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_kpi_data_points_kpi ON _atlas.kpi_data_points(kpi_id);
CREATE INDEX IF NOT EXISTS idx_kpi_data_points_recorded ON _atlas.kpi_data_points(kpi_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_kpi_data_points_org ON _atlas.kpi_data_points(organization_id);

-- KPI Dashboards
CREATE TABLE IF NOT EXISTS _atlas.kpi_dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    owner_id UUID,
    is_shared BOOLEAN NOT NULL DEFAULT false,
    is_default BOOLEAN NOT NULL DEFAULT false,
    layout_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, code)
);

-- Dashboard Widgets
CREATE TABLE IF NOT EXISTS _atlas.kpi_dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES _atlas.kpi_dashboards(id) ON DELETE CASCADE,
    kpi_id UUID REFERENCES _atlas.kpi_definitions(id) ON DELETE SET NULL,
    widget_type VARCHAR(30) NOT NULL DEFAULT 'kpi_card',
    title VARCHAR(200) NOT NULL,
    position_row INT NOT NULL DEFAULT 0,
    position_col INT NOT NULL DEFAULT 0,
    width INT NOT NULL DEFAULT 1,
    height INT NOT NULL DEFAULT 1,
    display_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_visible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_kpi_widgets_dashboard ON _atlas.kpi_dashboard_widgets(dashboard_id);
