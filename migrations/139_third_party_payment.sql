-- 139_third_party_payment.sql
-- Oracle Fusion Financial Feature: Third-Party Payment Management
-- Manages payments to third parties (tax authorities, garnishment recipients,
-- insurance providers, court-ordered deductions) on behalf of suppliers or employees.
-- Oracle Fusion: Financials > Payables > Third-Party Payments

-- ============================================================================
-- Third-Party Payment Header
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.third_party_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_number VARCHAR(50) NOT NULL,
    payment_type VARCHAR(30) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    source_entity_type VARCHAR(30) NOT NULL,
    source_entity_id UUID NOT NULL,
    source_entity_name VARCHAR(200) NOT NULL,
    payee_name VARCHAR(200) NOT NULL,
    payee_tax_id VARCHAR(50),
    payee_address TEXT,
    payee_bank_account VARCHAR(100),
    amount NUMERIC(18,2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_method VARCHAR(30),
    payment_date DATE,
    due_date DATE,
    reference_document VARCHAR(200),
    reference_document_id UUID,
    description TEXT,
    case_number VARCHAR(100),
    court_jurisdiction VARCHAR(200),
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    recurrence_frequency VARCHAR(20),
    recurrence_start_date DATE,
    recurrence_end_date DATE,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    hold_reason TEXT,
    hold_at TIMESTAMPTZ,
    hold_by UUID,
    payment_reference VARCHAR(100),
    paid_at TIMESTAMPTZ,
    paid_by UUID,
    cancellation_reason TEXT,
    cancelled_at TIMESTAMPTZ,
    cancelled_by UUID,
    rejection_reason TEXT,
    rejected_by UUID,
    rejected_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, payment_number)
);

CREATE INDEX idx_tpp_org ON financials.third_party_payments(organization_id);
CREATE INDEX idx_tpp_status ON financials.third_party_payments(status);
CREATE INDEX idx_tpp_type ON financials.third_party_payments(payment_type);
CREATE INDEX idx_tpp_source ON financials.third_party_payments(source_entity_type, source_entity_id);
CREATE INDEX idx_tpp_payee ON financials.third_party_payments(payee_name);
CREATE INDEX idx_tpp_due_date ON financials.third_party_payments(due_date) WHERE status IN ('draft', 'submitted', 'approved');
CREATE INDEX idx_tpp_recurring ON financials.third_party_payments(is_recurring) WHERE is_recurring = true;

-- ============================================================================
-- Third-Party Payment Lines (breakdown of payment amount by GL account)
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.third_party_payment_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_id UUID NOT NULL REFERENCES financials.third_party_payments(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    line_type VARCHAR(30) NOT NULL,
    description TEXT,
    amount NUMERIC(18,2) NOT NULL,
    gl_account VARCHAR(500),
    cost_center VARCHAR(50),
    tax_code VARCHAR(50),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tppl_payment ON financials.third_party_payment_lines(payment_id);
CREATE INDEX idx_tppl_type ON financials.third_party_payment_lines(line_type);

-- ============================================================================
-- Third-Party Payment Audit Trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS financials.third_party_payment_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    payment_id UUID NOT NULL REFERENCES financials.third_party_payments(id),
    payment_number VARCHAR(50) NOT NULL,
    action VARCHAR(30) NOT NULL,
    from_status VARCHAR(20),
    to_status VARCHAR(20),
    performed_by UUID,
    performed_at TIMESTAMPTZ DEFAULT now(),
    reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_tppa_payment ON financials.third_party_payment_audit(payment_id);
CREATE INDEX idx_tppa_action ON financials.third_party_payment_audit(action);
CREATE INDEX idx_tppa_performed ON financials.third_party_payment_audit(performed_at);

-- ============================================================================
-- Dashboard View
-- ============================================================================

CREATE OR REPLACE VIEW financials.third_party_payment_dashboard AS
SELECT
    tpp.organization_id,
    COUNT(*) AS total_payments,
    COUNT(*) FILTER (WHERE tpp.status = 'draft') AS draft_count,
    COUNT(*) FILTER (WHERE tpp.status = 'submitted') AS pending_approval_count,
    COUNT(*) FILTER (WHERE tpp.status = 'approved') AS approved_count,
    COUNT(*) FILTER (WHERE tpp.status = 'paid') AS paid_count,
    COUNT(*) FILTER (WHERE tpp.status = 'on_hold') AS on_hold_count,
    COUNT(*) FILTER (WHERE tpp.status = 'cancelled') AS cancelled_count,
    COALESCE(SUM(tpp.amount), 0) AS total_amount,
    COALESCE(SUM(tpp.amount) FILTER (WHERE tpp.status = 'paid'), 0) AS paid_amount,
    COALESCE(SUM(tpp.amount) FILTER (WHERE tpp.status IN ('draft', 'submitted', 'approved')), 0) AS pending_amount,
    COALESCE(SUM(tpp.amount) FILTER (WHERE tpp.status = 'on_hold'), 0) AS on_hold_amount,
    COALESCE(
        json_agg(DISTINCT jsonb_build_object(
            'payment_type', tpp.payment_type,
            'count', tpp_by_type.cnt,
            'total_amount', tpp_by_type.total
        )) FILTER (WHERE tpp_by_type.cnt IS NOT NULL),
        '[]'::json
    ) AS payments_by_type,
    COALESCE(
        (SELECT json_agg(jsonb_build_object(
            'id', upcoming.id,
            'payment_number', upcoming.payment_number,
            'payee_name', upcoming.payee_name,
            'amount', upcoming.amount,
            'due_date', upcoming.due_date,
            'payment_type', upcoming.payment_type
        ) ORDER BY upcoming.due_date) FROM financials.third_party_payments upcoming
        WHERE upcoming.organization_id = tpp.organization_id
          AND upcoming.status IN ('draft', 'submitted', 'approved')
          AND upcoming.due_date IS NOT NULL
          AND upcoming.due_date <= CURRENT_DATE + INTERVAL '30 days'),
        '[]'::json
    ) AS upcoming_due
FROM financials.third_party_payments tpp
LEFT JOIN LATERAL (
    SELECT COUNT(*) AS cnt, SUM(amount) AS total
    FROM financials.third_party_payments t2
    WHERE t2.organization_id = tpp.organization_id
      AND t2.payment_type = tpp.payment_type
) tpp_by_type ON true
GROUP BY tpp.organization_id;

COMMENT ON TABLE financials.third_party_payments IS 'Third-party payment headers - payments to third parties on behalf of suppliers or employees';
COMMENT ON TABLE financials.third_party_payment_lines IS 'Third-party payment lines - GL account breakdown of payment amounts';
COMMENT ON TABLE financials.third_party_payment_audit IS 'Audit trail for third-party payment state transitions';
COMMENT ON VIEW financials.third_party_payment_dashboard IS 'Dashboard aggregation for third-party payment statistics';
