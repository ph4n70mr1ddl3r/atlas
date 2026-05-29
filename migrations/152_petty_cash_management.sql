-- 152_petty_cash_management.sql
-- Oracle Fusion Financial Feature: Petty Cash Management
-- Manages petty cash funds, custodians, and disbursements.

CREATE TABLE IF NOT EXISTS financials.petty_cash_funds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    fund_name VARCHAR(100) NOT NULL,
    custodian_id UUID NOT NULL,
    location VARCHAR(200),
    currency_code VARCHAR(3) NOT NULL DEFAULT 'USD',
    authorized_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    current_balance DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, fund_name)
);

CREATE TABLE IF NOT EXISTS financials.petty_cash_disbursements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fund_id UUID NOT NULL REFERENCES financials.petty_cash_funds(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    receipt_reference VARCHAR(100),
    expense_category VARCHAR(50),
    description TEXT,
    disbursement_date DATE NOT NULL DEFAULT CURRENT_DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'CLEARED',
    created_at TIMESTAMPTZ DEFAULT now()
);

COMMENT ON TABLE financials.petty_cash_funds IS 'Petty Cash Funds - manages authorized petty cash pools and their custodians';
COMMENT ON TABLE financials.petty_cash_disbursements IS 'Petty Cash Disbursements - records individual payouts from a petty cash fund';
