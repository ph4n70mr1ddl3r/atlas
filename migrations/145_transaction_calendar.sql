-- 145_transaction_calendar.sql
-- Oracle Fusion Cloud ERP Feature: Transaction Calendars
-- Defines business calendars with working days, holidays, and exception dates.
-- Used throughout financials for date calculations: due dates, discount dates,
-- payment dates, posting validation, and cash forecasting.
-- Oracle Fusion: General Ledger > Setup > Transaction Calendars

-- ============================================================================
-- Transaction Calendars
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.transaction_calendars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Working days stored as a JSON array of ISO weekday numbers (1=Monday, 7=Sunday)
    -- e.g. [1,2,3,4,5] for Mon-Fri
    working_days JSONB NOT NULL DEFAULT '[1,2,3,4,5]'::jsonb,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    effective_from DATE,
    effective_to DATE,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_txncal_org ON _atlas.transaction_calendars(organization_id);
CREATE INDEX idx_txncal_status ON _atlas.transaction_calendars(status);

-- ============================================================================
-- Calendar Exceptions (holidays, non-working days, special working days)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.calendar_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    calendar_id UUID NOT NULL REFERENCES _atlas.transaction_calendars(id) ON DELETE CASCADE,
    exception_date DATE NOT NULL,
    exception_type VARCHAR(20) NOT NULL, -- 'holiday', 'non_working', 'special_working'
    name VARCHAR(200) NOT NULL,
    description TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(calendar_id, exception_date)
);

CREATE INDEX idx_cal_exc_cal ON _atlas.calendar_exceptions(calendar_id);
CREATE INDEX idx_cal_exc_date ON _atlas.calendar_exceptions(exception_date);
CREATE INDEX idx_cal_exc_type ON _atlas.calendar_exceptions(exception_type);

-- ============================================================================
-- Calendar Usage Audit (tracks date calculations performed)
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.calendar_date_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    calendar_id UUID NOT NULL,
    calendar_code VARCHAR(100) NOT NULL,
    operation VARCHAR(50) NOT NULL, -- 'next_business_day', 'previous_business_day', 'is_business_day', 'add_business_days'
    input_date DATE NOT NULL,
    result_date DATE,
    result_boolean BOOLEAN,
    business_days_added INT,
    reference_type VARCHAR(100), -- e.g. 'payment_terms', 'posting_date', 'cash_forecast'
    reference_id UUID,
    calculated_at TIMESTAMPTZ DEFAULT now(),
    calculated_by UUID
);

CREATE INDEX idx_calcalc_cal ON _atlas.calendar_date_calculations(calendar_id);
CREATE INDEX idx_calcalc_date ON _atlas.calendar_date_calculations(calculated_at);

COMMENT ON TABLE _atlas.transaction_calendars IS 'Defines business calendars with working day patterns for date calculations';
COMMENT ON TABLE _atlas.calendar_exceptions IS 'Holidays, non-working days, and special working days for transaction calendars';
COMMENT ON TABLE _atlas.calendar_date_calculations IS 'Audit log of business day calculations performed using transaction calendars';
