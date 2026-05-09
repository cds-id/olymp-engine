-- olymp-monitoring migrations
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    action TEXT NOT NULL,
    actor_id UUID,
    changes JSONB,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE exam_progress (
    id UUID PRIMARY KEY,
    exam_session_id UUID NOT NULL,
    participant_id UUID NOT NULL,
    questions_answered INTEGER NOT NULL,
    total_questions INTEGER NOT NULL,
    last_activity TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at);
CREATE INDEX idx_exam_progress_session ON exam_progress(exam_session_id);
