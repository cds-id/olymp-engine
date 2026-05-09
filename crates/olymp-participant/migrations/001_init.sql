-- olymp-participant: participants + participant_stages

CREATE TABLE participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    event_id UUID NOT NULL REFERENCES events(id),
    education_level_id UUID NOT NULL REFERENCES education_levels(id),
    subject_id UUID NOT NULL REFERENCES subjects(id),
    school_name TEXT,
    district_id UUID REFERENCES districts(id),
    province_id UUID REFERENCES provinces(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, event_id, subject_id)
);

CREATE TABLE participant_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    participant_id UUID NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
    stage_id UUID NOT NULL REFERENCES stages(id),
    status TEXT NOT NULL DEFAULT 'registered',
    score FLOAT8,
    completion_time_secs INT,
    rank INT,
    cheating_log_count INT NOT NULL DEFAULT 0,
    promoted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(participant_id, stage_id)
);

CREATE INDEX idx_participants_user ON participants(user_id);
CREATE INDEX idx_participants_event ON participants(event_id);
CREATE INDEX idx_participants_district ON participants(district_id);
CREATE INDEX idx_participants_province ON participants(province_id);
CREATE INDEX idx_ps_participant ON participant_stages(participant_id);
CREATE INDEX idx_ps_stage ON participant_stages(stage_id);
CREATE INDEX idx_ps_status ON participant_stages(status);
