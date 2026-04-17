-- Atlas ERP - Collections & Credit Management
-- Oracle Fusion Cloud ERP: Financials > Collections > Collections Management
--
-- Manages customer credit profiles, credit limits, credit scoring,
-- receivables aging analysis, collection strategies, dunning campaigns,
-- customer interactions/calls, promise-to-pay tracking, and write-offs.
--
-- This is a standard Oracle Fusion Cloud ERP feature for managing
-- overdue receivables and controlling customer credit exposure.

-- ============================================================================
-- Customer Credit Profiles
-- Oracle Fusion: Collections > Customer Credit Profiles
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.customer_credit_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Credit limit information
    credit_limit NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit_used NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit_available NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Credit scoring
    -- Risk classification: 'low', 'medium', 'high', 'very_high', 'defaulted'
    risk_classification VARCHAR(20) NOT NULL DEFAULT 'medium',
    -- Internal credit score (0-1000, higher = better)
    credit_score INT,
    -- External credit rating (e.g., D&B, Experian)
    external_credit_rating VARCHAR(20),
    external_rating_agency VARCHAR(50),
    external_rating_date DATE,
    -- Payment behavior
    -- Payment terms: 'net_15', 'net_30', 'net_45', 'net_60', 'due_on_receipt', 'cod'
    payment_terms VARCHAR(20) NOT NULL DEFAULT 'net_30',
    -- Average days to pay (calculated from payment history)
    average_days_to_pay NUMERIC(8,2),
    -- Number of overdue invoices (calculated)
    overdue_invoice_count INT NOT NULL DEFAULT 0,
    -- Total overdue amount (calculated)
    total_overdue_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Oldest overdue date
    oldest_overdue_date DATE,
    -- Credit hold
    -- Whether the customer is on credit hold (blocks new sales orders)
    credit_hold BOOLEAN NOT NULL DEFAULT false,
    credit_hold_reason TEXT,
    credit_hold_date TIMESTAMPTZ,
    credit_hold_by UUID,
    -- Last review
    last_review_date DATE,
    next_review_date DATE,
    -- Status: 'active', 'inactive', 'blocked'
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, customer_id)
);

CREATE INDEX idx_credit_profiles_org ON _atlas.customer_credit_profiles(organization_id);
CREATE INDEX idx_credit_profiles_customer ON _atlas.customer_credit_profiles(customer_id);
CREATE INDEX idx_credit_profiles_risk ON _atlas.customer_credit_profiles(organization_id, risk_classification);
CREATE INDEX idx_credit_profiles_hold ON _atlas.customer_credit_profiles(credit_hold) WHERE credit_hold = true;

-- ============================================================================
-- Collection Strategies
-- Oracle Fusion: Collections > Collection Strategies
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.collection_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Strategy identification
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Strategy type: 'automatic', 'manual'
    strategy_type VARCHAR(20) NOT NULL DEFAULT 'automatic',
    -- Applicable risk classifications (JSON array of strings)
    applicable_risk_classifications JSONB DEFAULT '["medium", "high", "very_high"]',
    -- Aging buckets that trigger this strategy (JSON array of bucket names)
    -- Buckets: 'current', '1_30', '31_60', '61_90', '91_120', '121_plus'
    trigger_aging_buckets JSONB DEFAULT '["31_60"]',
    -- Overdue amount threshold to trigger
    overdue_amount_threshold NUMERIC(18,2) DEFAULT 0,
    -- Collection actions (ordered list of steps)
    -- Each action: {"step": 1, "action_type": "call|email|letter|escalate|legal",
    --               "template": "...", "delay_days": 3, "assign_to_role": "..."}
    actions JSONB NOT NULL DEFAULT '[]',
    -- Priority (higher = more urgent)
    priority INT NOT NULL DEFAULT 50,
    -- Whether this strategy is active
    is_active BOOLEAN DEFAULT true,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, code)
);

CREATE INDEX idx_coll_strategies_org ON _atlas.collection_strategies(organization_id);
CREATE INDEX idx_coll_strategies_active ON _atlas.collection_strategies(organization_id, is_active) WHERE is_active = true;

-- ============================================================================
-- Collection Cases
-- Oracle Fusion: Collections > Collection Cases
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.collection_cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Case identification
    case_number VARCHAR(50) NOT NULL,
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Strategy reference
    strategy_id UUID REFERENCES _atlas.collection_strategies(id),
    -- Assigned collector
    assigned_to UUID,
    assigned_to_name VARCHAR(200),
    -- Case details
    -- 'collection', 'dispute', 'bankruptcy', 'skip_trace'
    case_type VARCHAR(20) NOT NULL DEFAULT 'collection',
    -- 'open', 'in_progress', 'resolved', 'closed', 'escalated', 'written_off'
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    -- Priority: 'low', 'medium', 'high', 'critical'
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    -- Financial summary at case creation
    total_overdue_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_disputed_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_invoiced_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    overdue_invoice_count INT NOT NULL DEFAULT 0,
    oldest_overdue_date DATE,
    -- Current action step in the strategy
    current_step INT NOT NULL DEFAULT 1,
    -- Key dates
    opened_date DATE NOT NULL DEFAULT CURRENT_DATE,
    target_resolution_date DATE,
    resolved_date DATE,
    closed_date DATE,
    last_action_date DATE,
    next_action_date DATE,
    -- Resolution
    resolution_type VARCHAR(30),
    -- 'full_payment', 'partial_payment', 'payment_plan', 'write_off',
    -- 'dispute_resolved', 'uncollectible', 'other'
    resolution_notes TEXT,
    -- Related invoices (JSON array of invoice IDs)
    related_invoice_ids JSONB DEFAULT '[]',
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, case_number)
);

CREATE INDEX idx_coll_cases_org ON _atlas.collection_cases(organization_id);
CREATE INDEX idx_coll_cases_customer ON _atlas.collection_cases(customer_id);
CREATE INDEX idx_coll_cases_status ON _atlas.collection_cases(organization_id, status);
CREATE INDEX idx_coll_cases_assigned ON _atlas.collection_cases(assigned_to) WHERE status IN ('open', 'in_progress');
CREATE INDEX idx_coll_cases_next_action ON _atlas.collection_cases(next_action_date) WHERE status IN ('open', 'in_progress');

-- ============================================================================
-- Customer Interactions (calls, emails, meetings)
-- Oracle Fusion: Collections > Customer Interactions
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.customer_interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Optional case reference
    case_id UUID REFERENCES _atlas.collection_cases(id),
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Interaction details
    -- 'phone_call', 'email', 'letter', 'meeting', 'note', 'sms'
    interaction_type VARCHAR(20) NOT NULL,
    -- Direction: 'outbound', 'inbound'
    direction VARCHAR(10) NOT NULL DEFAULT 'outbound',
    -- Who was contacted
    contact_name VARCHAR(200),
    contact_role VARCHAR(100),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(255),
    -- Interaction content
    subject VARCHAR(500),
    body TEXT,
    -- Result
    -- 'contacted', 'left_message', 'no_answer', 'promised_to_pay',
    -- 'disputed', 'refused', 'agreed_payment_plan', 'escalated', 'no_action'
    outcome VARCHAR(30),
    -- Follow-up
    follow_up_date DATE,
    follow_up_notes TEXT,
    -- Performed by
    performed_by UUID,
    performed_by_name VARCHAR(200),
    performed_at TIMESTAMPTZ DEFAULT now(),
    -- Duration (for calls, in minutes)
    duration_minutes INT,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_interactions_org ON _atlas.customer_interactions(organization_id);
CREATE INDEX idx_interactions_case ON _atlas.customer_interactions(case_id);
CREATE INDEX idx_interactions_customer ON _atlas.customer_interactions(customer_id);
CREATE INDEX idx_interactions_date ON _atlas.customer_interactions(performed_at);
CREATE INDEX idx_interactions_follow_up ON _atlas.customer_interactions(follow_up_date) WHERE follow_up_date IS NOT NULL;

-- ============================================================================
-- Promise to Pay
-- Oracle Fusion: Collections > Promises to Pay
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.promise_to_pay (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Optional case reference
    case_id UUID REFERENCES _atlas.collection_cases(id),
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Promise details
    -- 'single_payment', 'installment', 'full_balance'
    promise_type VARCHAR(20) NOT NULL DEFAULT 'single_payment',
    -- Total amount promised
    promised_amount NUMERIC(18,2) NOT NULL,
    -- Amount received against this promise
    paid_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Remaining amount
    remaining_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Promise dates
    promise_date DATE NOT NULL,
    -- For installments: number of installments and frequency
    installment_count INT,
    -- 'weekly', 'biweekly', 'monthly'
    installment_frequency VARCHAR(20),
    -- Status: 'pending', 'partially_kept', 'kept', 'broken', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Tracking
    broken_date DATE,
    broken_reason TEXT,
    -- Related invoices
    related_invoice_ids JSONB DEFAULT '[]',
    -- Who promised
    promised_by_name VARCHAR(200),
    promised_by_role VARCHAR(100),
    -- Notes
    notes TEXT,
    -- Recorded by
    recorded_by UUID,
    recorded_at TIMESTAMPTZ DEFAULT now(),
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_ptp_org ON _atlas.promise_to_pay(organization_id);
CREATE INDEX idx_ptp_case ON _atlas.promise_to_pay(case_id);
CREATE INDEX idx_ptp_customer ON _atlas.promise_to_pay(customer_id);
CREATE INDEX idx_ptp_status ON _atlas.promise_to_pay(organization_id, status);
CREATE INDEX idx_ptp_promise_date ON _atlas.promise_to_pay(promise_date);

-- ============================================================================
-- Dunning Campaigns
-- Oracle Fusion: Collections > Dunning Management
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.dunning_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Campaign identification
    campaign_number VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    -- Dunning level (escalating severity)
    -- 'reminder', 'first_notice', 'second_notice', 'final_notice', 'pre_legal', 'legal'
    dunning_level VARCHAR(20) NOT NULL DEFAULT 'reminder',
    -- Communication method
    -- 'email', 'letter', 'sms', 'phone'
    communication_method VARCHAR(20) NOT NULL DEFAULT 'email',
    -- Template reference (for email/letter content)
    template_id UUID,
    template_name VARCHAR(200),
    -- Target criteria
    -- Minimum overdue days to include
    min_overdue_days INT NOT NULL DEFAULT 1,
    -- Minimum overdue amount to include
    min_overdue_amount NUMERIC(18,2) DEFAULT 0,
    -- Target risk classifications (JSON array)
    target_risk_classifications JSONB DEFAULT '["medium", "high", "very_high"]',
    -- Exclude customers already in active collection cases
    exclude_active_cases BOOLEAN DEFAULT false,
    -- Campaign scheduling
    scheduled_date DATE,
    sent_date DATE,
    -- Counts
    target_customer_count INT NOT NULL DEFAULT 0,
    sent_count INT NOT NULL DEFAULT 0,
    failed_count INT NOT NULL DEFAULT 0,
    -- Status: 'draft', 'scheduled', 'in_progress', 'completed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, campaign_number)
);

CREATE INDEX idx_dunning_campaigns_org ON _atlas.dunning_campaigns(organization_id);
CREATE INDEX idx_dunning_campaigns_status ON _atlas.dunning_campaigns(organization_id, status);
CREATE INDEX idx_dunning_campaigns_scheduled ON _atlas.dunning_campaigns(scheduled_date) WHERE status = 'scheduled';

-- ============================================================================
-- Dunning Letters (individual letters sent per customer)
-- Oracle Fusion: Collections > Dunning Letters
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.dunning_letters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Campaign reference
    campaign_id UUID REFERENCES _atlas.dunning_campaigns(id),
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Customer address at time of sending
    customer_address JSONB,
    customer_email VARCHAR(255),
    -- Letter details
    dunning_level VARCHAR(20) NOT NULL,
    communication_method VARCHAR(20) NOT NULL,
    -- Amounts at time of generation
    total_overdue_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    overdue_invoice_count INT NOT NULL DEFAULT 0,
    oldest_overdue_date DATE,
    -- Aging breakdown at time of generation
    aging_current NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_1_30 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_31_60 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_61_90 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_91_120 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_121_plus NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Sending status
    -- 'pending', 'sent', 'delivered', 'bounced', 'failed', 'viewed'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    viewed_at TIMESTAMPTZ,
    failure_reason TEXT,
    -- Related invoices detail (JSON array of {invoice_number, amount, due_date, days_overdue})
    invoice_details JSONB DEFAULT '[]',
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_dunning_letters_org ON _atlas.dunning_letters(organization_id);
CREATE INDEX idx_dunning_letters_campaign ON _atlas.dunning_letters(campaign_id);
CREATE INDEX idx_dunning_letters_customer ON _atlas.dunning_letters(customer_id);
CREATE INDEX idx_dunning_letters_status ON _atlas.dunning_letters(organization_id, status);

-- ============================================================================
-- Receivables Aging Snapshot
-- Oracle Fusion: Collections > Aging Analysis
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.receivables_aging_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Snapshot date (typically end of day)
    snapshot_date DATE NOT NULL,
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Total outstanding
    total_outstanding NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Aging buckets
    aging_current NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_1_30 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_31_60 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_61_90 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_91_120 NUMERIC(18,2) NOT NULL DEFAULT 0,
    aging_121_plus NUMERIC(18,2) NOT NULL DEFAULT 0,
    -- Invoice counts per bucket
    count_current INT NOT NULL DEFAULT 0,
    count_1_30 INT NOT NULL DEFAULT 0,
    count_31_60 INT NOT NULL DEFAULT 0,
    count_61_90 INT NOT NULL DEFAULT 0,
    count_91_120 INT NOT NULL DEFAULT 0,
    count_121_plus INT NOT NULL DEFAULT 0,
    -- Calculated metrics
    -- Weighted average days overdue
    weighted_average_days_overdue NUMERIC(8,2) DEFAULT 0,
    -- Percentage overdue
    overdue_percent NUMERIC(5,2) DEFAULT 0,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_aging_snapshots_org ON _atlas.receivables_aging_snapshots(organization_id);
CREATE INDEX idx_aging_snapshots_date ON _atlas.receivables_aging_snapshots(snapshot_date);
CREATE INDEX idx_aging_snapshots_customer ON _atlas.receivables_aging_snapshots(customer_id, snapshot_date);

-- ============================================================================
-- Write-Off Requests
-- Oracle Fusion: Collections > Write-Off Management
-- ============================================================================

CREATE TABLE IF NOT EXISTS _atlas.write_off_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    -- Request identification
    request_number VARCHAR(50) NOT NULL,
    -- Customer reference
    customer_id UUID NOT NULL,
    customer_number VARCHAR(50),
    customer_name VARCHAR(200),
    -- Write-off type: 'bad_debt', 'small_balance', 'dispute', 'adjustment'
    write_off_type VARCHAR(20) NOT NULL,
    -- Amount to write off
    write_off_amount NUMERIC(18,2) NOT NULL,
    -- GL account for the write-off
    write_off_account_code VARCHAR(50),
    -- Reason
    reason TEXT NOT NULL,
    -- Related invoices (JSON array)
    related_invoice_ids JSONB DEFAULT '[]',
    -- Optional case reference
    case_id UUID REFERENCES _atlas.collection_cases(id),
    -- Status: 'draft', 'submitted', 'approved', 'rejected', 'processed', 'cancelled'
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    -- Approval workflow
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT,
    -- Journal entry reference (after posting)
    journal_entry_id UUID,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, request_number)
);

CREATE INDEX idx_write_offs_org ON _atlas.write_off_requests(organization_id);
CREATE INDEX idx_write_offs_customer ON _atlas.write_off_requests(customer_id);
CREATE INDEX idx_write_offs_status ON _atlas.write_off_requests(organization_id, status);
