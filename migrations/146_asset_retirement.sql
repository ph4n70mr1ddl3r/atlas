-- Asset Retirement / Disposal
-- Oracle Fusion Cloud ERP: Financials > Fixed Assets > Asset Retirements
-- Handles retirement of fixed assets through sale, scrap, donation, or transfer
-- with GL accounting entries for gain/loss on disposal.

CREATE TABLE IF NOT EXISTS _atlas.asset_retirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    retirement_number VARCHAR(50) NOT NULL,
    asset_id UUID NOT NULL,
    asset_number VARCHAR(100),
    asset_description TEXT,
    retirement_type VARCHAR(30) NOT NULL,  -- sale, scrap, donation, transfer, theft, destruction
    retirement_date DATE NOT NULL,
    cost VARCHAR(50) NOT NULL DEFAULT '0',
    accumulated_depreciation VARCHAR(50) NOT NULL DEFAULT '0',
    net_book_value VARCHAR(50) NOT NULL DEFAULT '0',
    proceeds VARCHAR(50) NOT NULL DEFAULT '0',
    removal_cost VARCHAR(50) NOT NULL DEFAULT '0',
    gain_loss_amount VARCHAR(50) NOT NULL DEFAULT '0',
    gain_loss_account VARCHAR(100),
    asset_account VARCHAR(100),
    depreciation_account VARCHAR(100),
    proceeds_account VARCHAR(100),
    removal_cost_account VARCHAR(100),
    buyer_name VARCHAR(200),
    reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, approved, completed, reversed, cancelled
    approved_by UUID,
    posted_to_gl BOOLEAN NOT NULL DEFAULT FALSE,
    gl_batch_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(organization_id, retirement_number)
);

CREATE INDEX IF NOT EXISTS idx_asset_retirements_org ON _atlas.asset_retirements(organization_id);
CREATE INDEX IF NOT EXISTS idx_asset_retirements_asset ON _atlas.asset_retirements(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_retirements_status ON _atlas.asset_retirements(status);
CREATE INDEX IF NOT EXISTS idx_asset_retirements_type ON _atlas.asset_retirements(retirement_type);
CREATE INDEX IF NOT EXISTS idx_asset_retirements_date ON _atlas.asset_retirements(retirement_date);
