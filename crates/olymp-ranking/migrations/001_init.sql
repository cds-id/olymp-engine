-- olymp-ranking migrations
CREATE TABLE scores (
    id UUID PRIMARY KEY,
    participant_id UUID NOT NULL,
    exam_id UUID NOT NULL,
    score FLOAT8 NOT NULL,
    graded_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE leaderboards (
    id UUID PRIMARY KEY,
    tier TEXT NOT NULL CHECK (tier IN ('district', 'province', 'national')),
    participant_id UUID NOT NULL,
    rank INTEGER NOT NULL,
    score FLOAT8 NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_scores_participant ON scores(participant_id);
CREATE INDEX idx_scores_exam ON scores(exam_id);
CREATE INDEX idx_leaderboards_tier ON leaderboards(tier);
CREATE INDEX idx_leaderboards_rank ON leaderboards(rank);
