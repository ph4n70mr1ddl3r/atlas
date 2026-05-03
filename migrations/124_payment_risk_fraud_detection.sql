-- Payment Risk & Fraud Detection
-- Oracle Fusion: Financials > Payables > Payment Risk Management
-- Manages payment risk profiling, fraud alerting, sanctions screening,
-- supplier risk assessment, duplicate payment detection, and velocity monitoring.
-- Uses VARCHAR for monetary amounts to avoid SQLx NUMERIC decoding issues.

-- Payment Risk Profile table
CREATE TABLE IF NOT EXISTS _atlas.payment_risk_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    profile_type VARCHAR(30) NOT NULL DEFAULT 'global',
    default_risk_level VARCHAR(20) NOT NULL DEFAULT 'medium',
    duplicate_amount_tolerance_pct VARCHAR(10) DEFAULT '5.00',
    duplicate_date_tolerance_days VARCHAR(10) DEFAULT '3.00',
    velocity_daily_limit VARCHAR(50),
    velocity_weekly_limit VARCHAR(50),
    amount_anomaly_std_dev VARCHAR(10) DEFAULT '2.00',
    enable_sanctions_screening BOOLEAN DEFAULT true,
    enable_duplicate_detection BOOLEAN DEFAULT true,
    enable_velocity_checks BOOLEAN DEFAULT true,
    enable_amount_anomaly BOOLEAN DEFAULT true,
    enable_behavioral_analysis BOOLEAN DEFAULT false,
    auto_block_critical BOOLEAN DEFAULT true,
    auto_block_high BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    effective_from DATE,
    effective_to DATE,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    UNIQUE(organization_id, code)
);

-- Payment Fraud Alert table
CREATE TABLE IF NOT EXISTS _atlas.payment_fraud_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    alert_number VARCHAR(50) NOT NULL,
    alert_type VARCHAR(30) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    payment_id UUID,
    invoice_id UUID,
    supplier_id UUID,
    supplier_number VARCHAR(50),
    supplier_name VARCHAR(300),
    amount VARCHAR(50),
    currency_code VARCHAR(3) DEFAULT 'USD',
    risk_score VARCHAR(10),
    detection_rule VARCHAR(100),
    description TEXT,
    evidence TEXT,
    assigned_to VARCHAR(200),
    assigned_team VARCHAR(200),
    detected_date DATE NOT NULL DEFAULT CURRENT_DATE,
    resolution_date DATE,
    resolution_notes TEXT,
    resolved_by UUID,
    related_alert_ids TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    UNIQUE(organization_id, alert_number)
);

-- Sanctions Screening Result table
CREATE TABLE IF NOT EXISTS _atlas.sanctions_screening_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    screening_id VARCHAR(50) NOT NULL,
    screening_type VARCHAR(30) NOT NULL,
    supplier_id UUID,
    supplier_name VARCHAR(300),
    payment_id UUID,
    screened_list VARCHAR(30) NOT NULL,
    match_name VARCHAR(300),
    match_type VARCHAR(20) NOT NULL DEFAULT 'none',
    match_score VARCHAR(10) DEFAULT '0.00',
    match_status VARCHAR(20) NOT NULL DEFAULT 'no_match',
    sanctions_list_entry VARCHAR(300),
    sanctions_list_program VARCHAR(100),
    match_details TEXT,
    reviewed_by VARCHAR(200),
    reviewed_date DATE,
    review_notes TEXT,
    action_taken VARCHAR(30) DEFAULT 'none',
    screening_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    UNIQUE(organization_id, screening_id)
);

-- Supplier Risk Assessment table
CREATE TABLE IF NOT EXISTS _atlas.supplier_risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    assessment_number VARCHAR(50) NOT NULL,
    supplier_id UUID NOT NULL,
    supplier_name VARCHAR(300) NOT NULL,
    assessment_date DATE NOT NULL DEFAULT CURRENT_DATE,
    assessment_type VARCHAR(20) NOT NULL DEFAULT 'periodic',
    overall_risk_level VARCHAR(20) NOT NULL DEFAULT 'medium',
    financial_risk_score VARCHAR(10) DEFAULT '0.00',
    operational_risk_score VARCHAR(10) DEFAULT '0.00',
    compliance_risk_score VARCHAR(10) DEFAULT '0.00',
    payment_history_score VARCHAR(10) DEFAULT '0.00',
    overall_risk_score VARCHAR(10) DEFAULT '0.00',
    years_in_business INTEGER,
    has_financial_statements BOOLEAN DEFAULT false,
    has_audit_reports BOOLEAN DEFAULT false,
    has_insurance BOOLEAN DEFAULT false,
    is_sanctions_clear BOOLEAN DEFAULT false,
    is_aml_clear BOOLEAN DEFAULT false,
    is_pep_clear BOOLEAN DEFAULT false,
    payment_behavior_rating VARCHAR(20) DEFAULT 'fair',
    total_historical_payments INTEGER DEFAULT 0,
    total_historical_amount VARCHAR(50) DEFAULT '0.00',
    fraud_alerts_count INTEGER DEFAULT 0,
    duplicate_payments_count INTEGER DEFAULT 0,
    assessed_by VARCHAR(200),
    findings TEXT,
    recommendations TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID,
    UNIQUE(organization_id, assessment_number)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_org_status ON _atlas.payment_fraud_alerts (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_supplier ON _atlas.payment_fraud_alerts (organization_id, supplier_id);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_type ON _atlas.payment_fraud_alerts (organization_id, alert_type);
CREATE INDEX IF NOT EXISTS idx_sanctions_supplier ON _atlas.sanctions_screening_results (organization_id, supplier_id);
CREATE INDEX IF NOT EXISTS idx_sanctions_status ON _atlas.sanctions_screening_results (organization_id, match_status);
CREATE INDEX IF NOT EXISTS idx_risk_assess_supplier ON _atlas.supplier_risk_assessments (organization_id, supplier_id);
CREATE INDEX IF NOT EXISTS idx_risk_assess_status ON _atlas.supplier_risk_assessments (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_risk_profiles_org ON _atlas.payment_risk_profiles (organization_id);
