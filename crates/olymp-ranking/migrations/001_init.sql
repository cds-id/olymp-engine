-- olymp-ranking: ranking_rules, ranking_results, ranking_entries

CREATE TABLE ranking_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_id UUID NOT NULL REFERENCES stages(id),
    max_qualifiers INT,
    min_score FLOAT8,
    max_cheating_logs INT,
    tiebreaker_order JSONB NOT NULL DEFAULT '["score_desc","time_asc","cheating_asc"]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(stage_id)
);

CREATE TABLE ranking_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_id UUID NOT NULL REFERENCES stages(id),
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft','reviewed','approved','published')),
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    total_participants INT NOT NULL,
    total_qualified INT NOT NULL DEFAULT 0
);

CREATE TABLE ranking_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ranking_result_id UUID NOT NULL REFERENCES ranking_results(id) ON DELETE CASCADE,
    participant_stage_id UUID NOT NULL REFERENCES participant_stages(id),
    rank INT NOT NULL,
    score FLOAT8 NOT NULL,
    completion_time_secs INT,
    cheating_log_count INT NOT NULL DEFAULT 0,
    qualification_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (qualification_status IN ('pending','qualified','not_qualified','disqualified')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ranking_rules_stage ON ranking_rules(stage_id);
CREATE INDEX idx_ranking_results_stage ON ranking_results(stage_id);
CREATE INDEX idx_ranking_results_status ON ranking_results(status);
CREATE INDEX idx_ranking_entries_result ON ranking_entries(ranking_result_id);
CREATE INDEX idx_ranking_entries_rank ON ranking_entries(ranking_result_id, rank);
CREATE INDEX idx_ranking_entries_ps ON ranking_entries(participant_stage_id);
