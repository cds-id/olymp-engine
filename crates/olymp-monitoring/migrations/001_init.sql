-- olymp-monitoring: cheating_logs, exam_progress, audit_logs
-- Ensure logs are immutable and retained per compliance requirements

CREATE TABLE cheating_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_session_id UUID NOT NULL REFERENCES exam_sessions(id),
    event_type TEXT NOT NULL,
    detail JSONB,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE exam_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_session_id UUID NOT NULL REFERENCES exam_sessions(id),
    questions_answered INT NOT NULL DEFAULT 0,
    total_questions INT NOT NULL,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(exam_session_id)
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID,
    event_id UUID,
    metadata JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cheating_logs_session ON cheating_logs(exam_session_id);
CREATE INDEX idx_cheating_logs_type ON cheating_logs(event_type);
CREATE INDEX idx_exam_progress_session ON exam_progress(exam_session_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_event ON audit_logs(event_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
