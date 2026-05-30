BEGIN;

CREATE TABLE _atlas.fin_fa_revaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_type_code VARCHAR(30) NOT NULL,
    description VARCHAR(255),
    revaluation_date DATE NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'PENDING',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE _atlas.fin_fa_revaluation_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    revaluation_id UUID NOT NULL REFERENCES _atlas.fin_fa_revaluations(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL,
    revaluation_rate NUMERIC,
    fair_value NUMERIC,
    old_cost NUMERIC,
    new_cost NUMERIC,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_fa_revaluations_book ON _atlas.fin_fa_revaluations(book_type_code);
CREATE INDEX idx_fa_revaluation_lines_rev ON _atlas.fin_fa_revaluation_lines(revaluation_id);

COMMENT ON TABLE _atlas.fin_fa_revaluations IS 'Fixed asset revaluation events for a corporate or tax book';
COMMENT ON TABLE _atlas.fin_fa_revaluation_lines IS 'Individual asset adjustments for a revaluation event';

COMMIT;
