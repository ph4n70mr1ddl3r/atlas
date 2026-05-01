-- 095: Funds Reservation & Budgetary Control
-- Oracle Fusion Cloud: Financials > Budgetary Control > Funds Reservation
-- Enables fund reservations, availability checks, consumption, and releases.

-- Fund Reservations
CREATE TABLE IF NOT EXISTS _atlas.fund_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    reservation_number VARCHAR(100) NOT NULL,
    budget_id UUID NOT NULL,
    budget_code VARCHAR(100) NOT NULL,
    budget_version_id UUID,
    description TEXT,
    source_type VARCHAR(50),
    source_id UUID,
    source_number VARCHAR(100),
    reserved_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    consumed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    released_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    remaining_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    currency_code VARCHAR(10) NOT NULL DEFAULT 'USD',
    reservation_date DATE NOT NULL,
    expiry_date DATE,
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    control_level VARCHAR(20) NOT NULL DEFAULT 'advisory',
    fiscal_year INT,
    period_name VARCHAR(50),
    department_id UUID,
    department_name VARCHAR(200),
    fund_check_passed BOOLEAN NOT NULL DEFAULT true,
    fund_check_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID,
    approved_by UUID,
    cancelled_by UUID,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_fund_reservations_number UNIQUE (organization_id, reservation_number),
    CONSTRAINT chk_fund_reservations_reserved_positive CHECK (reserved_amount >= 0),
    CONSTRAINT chk_fund_reservations_control_level CHECK (control_level IN ('advisory', 'absolute'))
);

CREATE INDEX IF NOT EXISTS idx_fund_reservations_org ON _atlas.fund_reservations (organization_id);
CREATE INDEX IF NOT EXISTS idx_fund_reservations_status ON _atlas.fund_reservations (organization_id, status);
CREATE INDEX IF NOT EXISTS idx_fund_reservations_budget ON _atlas.fund_reservations (organization_id, budget_id);
CREATE INDEX IF NOT EXISTS idx_fund_reservations_dept ON _atlas.fund_reservations (organization_id, department_id);
CREATE INDEX IF NOT EXISTS idx_fund_reservations_date ON _atlas.fund_reservations (reservation_date);

-- Fund Reservation Lines
CREATE TABLE IF NOT EXISTS _atlas.fund_reservation_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    reservation_id UUID NOT NULL REFERENCES _atlas.fund_reservations(id) ON DELETE CASCADE,
    line_number INT NOT NULL DEFAULT 1,
    account_code VARCHAR(200) NOT NULL,
    account_description VARCHAR(500),
    budget_line_id UUID,
    department_id UUID,
    project_id UUID,
    cost_center VARCHAR(100),
    reserved_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    consumed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    released_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    remaining_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_fund_reservation_lines_reserved CHECK (reserved_amount >= 0)
);

CREATE INDEX IF NOT EXISTS idx_fund_reservation_lines_reservation ON _atlas.fund_reservation_lines (reservation_id);
CREATE INDEX IF NOT EXISTS idx_fund_reservation_lines_account ON _atlas.fund_reservation_lines (organization_id, account_code);

COMMENT ON TABLE _atlas.fund_reservations IS 'Fund reservations for budgetary control - Oracle Fusion: Budgetary Control > Funds Reservation';
COMMENT ON TABLE _atlas.fund_reservation_lines IS 'Individual lines within fund reservations tied to specific accounts';
