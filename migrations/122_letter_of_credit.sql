-- Letter of Credit Management
-- Oracle Fusion: Treasury > Trade Finance > Letters of Credit
-- Manages import/export letters of credit, amendments, presentations,
-- shipments, and document tracking for international trade finance.
-- Uses VARCHAR for monetary amounts to avoid SQLx NUMERIC decoding issues.

-- Letter of Credit master table
CREATE TABLE IF NOT EXISTS _atlas.letters_of_credit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lc_number VARCHAR(50) NOT NULL,
    lc_type VARCHAR(30) NOT NULL DEFAULT 'import',
    lc_form VARCHAR(30) NOT NULL DEFAULT 'irrevocable',
    description TEXT,
    applicant_name VARCHAR(300) NOT NULL,
    applicant_address TEXT,
    applicant_bank_name VARCHAR(300) NOT NULL,
    applicant_bank_swift VARCHAR(20),
    beneficiary_name VARCHAR(300) NOT NULL,
    beneficiary_address TEXT,
    beneficiary_bank_name VARCHAR(300),
    beneficiary_bank_swift VARCHAR(20),
    advising_bank_name VARCHAR(300),
    advising_bank_swift VARCHAR(20),
    confirming_bank_name VARCHAR(300),
    confirming_bank_swift VARCHAR(20),
    lc_amount VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    tolerance_plus VARCHAR(10) DEFAULT '0',
    tolerance_minus VARCHAR(10) DEFAULT '0',
    available_with VARCHAR(300),
    available_by VARCHAR(30) NOT NULL DEFAULT 'payment',
    draft_at VARCHAR(100),
    issue_date DATE,
    expiry_date DATE NOT NULL,
    place_of_expiry VARCHAR(200),
    partial_shipments VARCHAR(20) DEFAULT 'allowed',
    transshipment VARCHAR(20) DEFAULT 'allowed',
    port_of_loading VARCHAR(200),
    port_of_discharge VARCHAR(200),
    shipment_period DATE,
    latest_shipment_date DATE,
    goods_description TEXT,
    incoterms VARCHAR(20),
    additional_conditions TEXT,
    bank_charges VARCHAR(30) DEFAULT 'beneficiary',
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    amendment_count INT DEFAULT 0,
    latest_amendment_number VARCHAR(50),
    reference_po_number VARCHAR(100),
    reference_contract_number VARCHAR(100),
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    approved_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, lc_number)
);

-- LC Amendments
CREATE TABLE IF NOT EXISTS _atlas.lc_amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lc_id UUID NOT NULL REFERENCES _atlas.letters_of_credit(id) ON DELETE CASCADE,
    lc_number VARCHAR(50) NOT NULL,
    amendment_number VARCHAR(50) NOT NULL,
    amendment_type VARCHAR(30) NOT NULL,
    previous_amount VARCHAR(50),
    new_amount VARCHAR(50),
    previous_expiry_date DATE,
    new_expiry_date DATE,
    previous_terms TEXT,
    new_terms TEXT,
    reason TEXT,
    bank_reference VARCHAR(100),
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    effective_date DATE,
    approved_by UUID,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- LC Required Documents
CREATE TABLE IF NOT EXISTS _atlas.lc_required_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lc_id UUID NOT NULL REFERENCES _atlas.letters_of_credit(id) ON DELETE CASCADE,
    document_type VARCHAR(100) NOT NULL,
    document_code VARCHAR(50),
    description TEXT,
    original_copies INT DEFAULT 1,
    copy_count INT DEFAULT 0,
    is_mandatory BOOLEAN DEFAULT true,
    special_instructions TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- LC Shipments
CREATE TABLE IF NOT EXISTS _atlas.lc_shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lc_id UUID NOT NULL REFERENCES _atlas.letters_of_credit(id) ON DELETE CASCADE,
    shipment_number VARCHAR(50) NOT NULL,
    vessel_name VARCHAR(200),
    voyage_number VARCHAR(50),
    bill_of_lading_number VARCHAR(100),
    carrier_name VARCHAR(200),
    port_of_loading VARCHAR(200),
    port_of_discharge VARCHAR(200),
    shipment_date DATE,
    expected_arrival_date DATE,
    actual_arrival_date DATE,
    shipping_marks TEXT,
    container_numbers TEXT,
    goods_description TEXT,
    quantity VARCHAR(50),
    unit_price VARCHAR(50),
    shipment_amount VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- LC Presentations (document submissions for payment)
CREATE TABLE IF NOT EXISTS _atlas.lc_presentations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    lc_id UUID NOT NULL REFERENCES _atlas.letters_of_credit(id) ON DELETE CASCADE,
    presentation_number VARCHAR(50) NOT NULL,
    shipment_id UUID REFERENCES _atlas.lc_shipments(id),
    presentation_date DATE NOT NULL,
    presenting_bank_name VARCHAR(300),
    total_amount VARCHAR(50) NOT NULL,
    currency_code VARCHAR(3) DEFAULT 'USD',
    document_count INT DEFAULT 0,
    discrepant BOOLEAN DEFAULT false,
    discrepancies TEXT,
    bank_response VARCHAR(30),
    response_date DATE,
    payment_due_date DATE,
    payment_date DATE,
    paid_amount VARCHAR(50),
    status VARCHAR(30) NOT NULL DEFAULT 'submitted',
    notes TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- LC Presentation Documents (actual submitted documents)
CREATE TABLE IF NOT EXISTS _atlas.lc_presentation_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    presentation_id UUID NOT NULL REFERENCES _atlas.lc_presentations(id) ON DELETE CASCADE,
    required_document_id UUID REFERENCES _atlas.lc_required_documents(id),
    document_type VARCHAR(100) NOT NULL,
    document_reference VARCHAR(100),
    description TEXT,
    original_copies INT DEFAULT 1,
    copy_count INT DEFAULT 0,
    is_compliant BOOLEAN DEFAULT true,
    discrepancies TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- LC Dashboard cache
CREATE TABLE IF NOT EXISTS _atlas.lc_dashboard_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    total_active_lcs INT DEFAULT 0,
    total_lc_amount VARCHAR(50) DEFAULT '0',
    total_pending_amendments INT DEFAULT 0,
    total_presentations_pending INT DEFAULT 0,
    total_discrepant_presentations INT DEFAULT 0,
    expiring_within_30_days INT DEFAULT 0,
    expiring_within_90_days INT DEFAULT 0,
    by_type JSONB DEFAULT '{}',
    by_currency JSONB DEFAULT '{}',
    by_status JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(organization_id, snapshot_date)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_lc_org_status ON _atlas.letters_of_credit(organization_id, status);
CREATE INDEX IF NOT EXISTS idx_lc_org_number ON _atlas.letters_of_credit(organization_id, lc_number);
CREATE INDEX IF NOT EXISTS idx_lc_amendments_lc ON _atlas.lc_amendments(lc_id);
CREATE INDEX IF NOT EXISTS idx_lc_required_docs_lc ON _atlas.lc_required_documents(lc_id);
CREATE INDEX IF NOT EXISTS idx_lc_shipments_lc ON _atlas.lc_shipments(lc_id);
CREATE INDEX IF NOT EXISTS idx_lc_presentations_lc ON _atlas.lc_presentations(lc_id);
CREATE INDEX IF NOT EXISTS idx_lc_pres_docs_pres ON _atlas.lc_presentation_documents(presentation_id);
CREATE INDEX IF NOT EXISTS idx_lc_dashboard_org ON _atlas.lc_dashboard_cache(organization_id);
