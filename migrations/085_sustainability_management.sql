-- Sustainability & ESG Management
-- Oracle Fusion Cloud: Sustainability / Environmental Accounting and Reporting
-- Provides: GHG emissions tracking (Scope 1/2/3), energy consumption,
--   waste management, water usage, sustainability metrics, ESG goals,
--   carbon offset management, sustainability dashboard.
--
-- Inspired by Oracle Fusion Cloud Sustainability and ESG reporting frameworks
-- (GRI, SASB, TCFD, EU Taxonomy).

-- ============================================================================
-- Sustainability Facilities: sites / buildings being tracked
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.sustainability_facilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    facility_code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Location
    country_code VARCHAR(10),
    region VARCHAR(200),
    city VARCHAR(200),
    address TEXT,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    -- Classification
    facility_type VARCHAR(50) NOT NULL DEFAULT 'office',  -- office, manufacturing, warehouse, data_center, retail, other
    industry_sector VARCHAR(200),
    total_area_sqm DOUBLE PRECISION,
    employee_count INT,
    operating_hours_per_year INT DEFAULT 8760,
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, facility_code)
);

CREATE INDEX IF NOT EXISTS idx_sust_facilities_org ON _atlas.sustainability_facilities(organization_id);
CREATE INDEX IF NOT EXISTS idx_sust_facilities_status ON _atlas.sustainability_facilities(status);

-- ============================================================================
-- Emission Factors: conversion factors for CO2e calculations
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.emission_factors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    factor_code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Scope: 1=direct, 2=indirect_energy, 3=other_indirect
    scope VARCHAR(30) NOT NULL,
    -- Category within scope (e.g., stationary_combustion, mobile_combustion, purchased_electricity)
    category VARCHAR(100) NOT NULL,
    -- Source activity (e.g., natural_gas, diesel, electricity_grid)
    activity_type VARCHAR(200) NOT NULL,
    -- Factor value: kg CO2e per unit
    factor_value DOUBLE PRECISION NOT NULL,
    -- Unit of the activity (e.g., kWh, liters, kg, therms, gallons)
    unit_of_measure VARCHAR(50) NOT NULL,
    -- Gas type: co2, ch4, n2o, hfcs, pfcs, sf6, co2e
    gas_type VARCHAR(20) NOT NULL DEFAULT 'co2e',
    -- Source of factor (e.g., EPA, DEFRA, IPCC)
    factor_source VARCHAR(200),
    -- Effectivity
    effective_from DATE NOT NULL,
    effective_to DATE,
    -- Geographic applicability
    region_code VARCHAR(50),
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, factor_code)
);

CREATE INDEX IF NOT EXISTS idx_emission_factors_org ON _atlas.emission_factors(organization_id);
CREATE INDEX IF NOT EXISTS idx_emission_factors_scope ON _atlas.emission_factors(scope);
CREATE INDEX IF NOT EXISTS idx_emission_factors_activity ON _atlas.emission_factors(activity_type);

-- ============================================================================
-- Environmental Activity Logs: raw consumption/emission data entries
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.environmental_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    activity_number VARCHAR(100) NOT NULL,
    -- What facility
    facility_id UUID REFERENCES _atlas.sustainability_facilities(id),
    facility_code VARCHAR(100),
    -- Activity classification
    activity_type VARCHAR(200) NOT NULL,    -- natural_gas, electricity, diesel, gasoline, water, waste, etc.
    scope VARCHAR(30) NOT NULL,             -- scope_1, scope_2, scope_3
    category VARCHAR(100),
    -- Measurement
    quantity DOUBLE PRECISION NOT NULL,
    unit_of_measure VARCHAR(50) NOT NULL,
    -- Emission factor used
    emission_factor_id UUID REFERENCES _atlas.emission_factors(id),
    -- Computed emissions
    co2e_kg DOUBLE PRECISION NOT NULL DEFAULT 0,
    co2_kg DOUBLE PRECISION DEFAULT 0,
    ch4_kg DOUBLE PRECISION DEFAULT 0,
    n2o_kg DOUBLE PRECISION DEFAULT 0,
    -- Cost
    cost_amount DOUBLE PRECISION,
    cost_currency VARCHAR(10) DEFAULT 'USD',
    -- Time period
    activity_date DATE NOT NULL,
    reporting_period VARCHAR(50),           -- e.g., "2024-Q1", "2024-H1", "2024"
    -- Source reference (GL journal, invoice, etc.)
    source_type VARCHAR(100),               -- manual_entry, gl_journal, invoice, meter_reading, estimation
    source_reference VARCHAR(200),
    -- Optional project / department attribution
    department_id UUID,
    project_id UUID,
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'confirmed',  -- draft, confirmed, verified, adjusted
    -- Verification
    verified_by UUID,
    verified_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, activity_number)
);

CREATE INDEX IF NOT EXISTS idx_env_activities_org ON _atlas.environmental_activities(organization_id);
CREATE INDEX IF NOT EXISTS idx_env_activities_facility ON _atlas.environmental_activities(facility_id);
CREATE INDEX IF NOT EXISTS idx_env_activities_scope ON _atlas.environmental_activities(scope);
CREATE INDEX IF NOT EXISTS idx_env_activities_date ON _atlas.environmental_activities(activity_date);
CREATE INDEX IF NOT EXISTS idx_env_activities_period ON _atlas.environmental_activities(reporting_period);
CREATE INDEX IF NOT EXISTS idx_env_activities_type ON _atlas.environmental_activities(activity_type);

-- ============================================================================
-- ESG Metrics: track key ESG performance indicators
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.esg_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    metric_code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- ESG pillar
    pillar VARCHAR(20) NOT NULL,            -- environmental, social, governance
    -- Category within pillar (e.g., climate, energy, water, waste, diversity, safety, ethics)
    category VARCHAR(100) NOT NULL,
    -- Measurement
    unit_of_measure VARCHAR(50) NOT NULL,
    -- Reporting framework alignment
    gri_standard VARCHAR(100),
    sasb_standard VARCHAR(100),
    tcfd_category VARCHAR(100),
    eu_taxonomy_code VARCHAR(100),
    -- Target / threshold
    target_value DOUBLE PRECISION,
    warning_threshold DOUBLE PRECISION,
    -- Higher is better or lower is better
    direction VARCHAR(20) NOT NULL DEFAULT 'lower_is_better',  -- lower_is_better, higher_is_better
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, metric_code)
);

CREATE INDEX IF NOT EXISTS idx_esg_metrics_org ON _atlas.esg_metrics(organization_id);
CREATE INDEX IF NOT EXISTS idx_esg_metrics_pillar ON _atlas.esg_metrics(pillar);

-- ============================================================================
-- ESG Metric Readings: actual values over time
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.esg_metric_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    metric_id UUID NOT NULL REFERENCES _atlas.esg_metrics(id) ON DELETE CASCADE,
    -- Value
    metric_value DOUBLE PRECISION NOT NULL,
    -- Time
    reading_date DATE NOT NULL,
    reporting_period VARCHAR(50),
    -- Facility-level granularity (optional)
    facility_id UUID REFERENCES _atlas.sustainability_facilities(id),
    -- Context
    notes TEXT,
    source VARCHAR(100),                    -- manual, automated, calculated, third_party
    verified_by UUID,
    verified_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'confirmed',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_esg_readings_metric ON _atlas.esg_metric_readings(metric_id);
CREATE INDEX IF NOT EXISTS idx_esg_readings_org ON _atlas.esg_metric_readings(organization_id);
CREATE INDEX IF NOT EXISTS idx_esg_readings_date ON _atlas.esg_metric_readings(reading_date);

-- ============================================================================
-- Sustainability Goals: organizational targets for sustainability
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.sustainability_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    goal_code VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Goal type
    goal_type VARCHAR(50) NOT NULL,         -- emission_reduction, energy_efficiency, waste_reduction, water_reduction, carbon_neutral, renewable_energy
    -- Scope
    scope VARCHAR(30),                      -- scope_1, scope_2, scope_3, all_scopes (for emission goals)
    -- Baseline
    baseline_value DOUBLE PRECISION NOT NULL,
    baseline_year INT NOT NULL,
    baseline_unit VARCHAR(50) NOT NULL,
    -- Target
    target_value DOUBLE PRECISION NOT NULL,
    target_year INT NOT NULL,
    target_unit VARCHAR(50) NOT NULL,
    -- Reduction / improvement percentage
    target_reduction_pct DOUBLE PRECISION,
    -- Interim milestones (JSON array of {year, value})
    milestones JSONB DEFAULT '[]'::jsonb,
    -- Current progress
    current_value DOUBLE PRECISION DEFAULT 0,
    progress_pct DOUBLE PRECISION DEFAULT 0,
    -- Facility / org-wide
    facility_id UUID REFERENCES _atlas.sustainability_facilities(id),
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'on_track',  -- on_track, at_risk, off_track, achieved, cancelled
    -- Owner
    owner_id UUID,
    owner_name VARCHAR(300),
    -- Framework alignment
    framework VARCHAR(100),                 -- SBTi, UN_SDG, Paris_Agreement, Custom
    framework_reference VARCHAR(200),
    -- Dates
    effective_from DATE,
    effective_to DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, goal_code)
);

CREATE INDEX IF NOT EXISTS idx_sust_goals_org ON _atlas.sustainability_goals(organization_id);
CREATE INDEX IF NOT EXISTS idx_sust_goals_type ON _atlas.sustainability_goals(goal_type);
CREATE INDEX IF NOT EXISTS idx_sust_goals_status ON _atlas.sustainability_goals(status);

-- ============================================================================
-- Carbon Offsets: purchased carbon credits / offsets
-- ============================================================================
CREATE TABLE IF NOT EXISTS _atlas.carbon_offsets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    offset_number VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    -- Offset project details
    project_name VARCHAR(500) NOT NULL,
    project_type VARCHAR(100) NOT NULL,     -- reforestation, renewable_energy, methane_capture, direct_air_capture, cookstove, other
    project_location VARCHAR(300),
    -- Certification
    registry VARCHAR(100),                  -- Verra, Gold_Standard, ACR, CAR
    registry_id VARCHAR(200),
    certification_standard VARCHAR(200),
    -- Quantity
    quantity_tonnes DOUBLE PRECISION NOT NULL,
    remaining_tonnes DOUBLE PRECISION NOT NULL,
    unit_price DOUBLE PRECISION,
    total_cost DOUBLE PRECISION,
    currency_code VARCHAR(10) DEFAULT 'USD',
    -- Vintage
    vintage_year INT NOT NULL,
    -- Retirement
    retired_quantity DOUBLE PRECISION DEFAULT 0,
    retired_date DATE,
    -- Effectivity
    effective_from DATE NOT NULL,
    effective_to DATE,
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active',  -- active, retired, expired, cancelled
    -- Counterparty
    supplier_name VARCHAR(300),
    supplier_id UUID,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, offset_number)
);

CREATE INDEX IF NOT EXISTS idx_carbon_offsets_org ON _atlas.carbon_offsets(organization_id);
CREATE INDEX IF NOT EXISTS idx_carbon_offsets_status ON _atlas.carbon_offsets(status);
CREATE INDEX IF NOT EXISTS idx_carbon_offsets_type ON _atlas.carbon_offsets(project_type);
