-- olymp-participant migrations
CREATE TABLE participants (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE,
    current_tier TEXT NOT NULL CHECK (current_tier IN ('district', 'province', 'national')),
    is_locked BOOLEAN NOT NULL DEFAULT false,
    locked_by_account TEXT,
    score FLOAT8 NOT NULL DEFAULT 0.0,
    rank INTEGER,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE tier_progressions (
    id UUID PRIMARY KEY,
    participant_id UUID NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
    from_tier TEXT NOT NULL,
    to_tier TEXT NOT NULL,
    advanced_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_participants_user_id ON participants(user_id);
CREATE INDEX idx_participants_tier ON participants(current_tier);
CREATE INDEX idx_tier_progressions_participant ON tier_progressions(participant_id);
