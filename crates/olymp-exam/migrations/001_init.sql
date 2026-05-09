-- olymp-exam migrations
CREATE TABLE exams (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    tier TEXT NOT NULL CHECK (tier IN ('district', 'province', 'national')),
    duration_minutes INTEGER NOT NULL,
    total_questions INTEGER NOT NULL,
    passing_score FLOAT8 NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE questions (
    id UUID PRIMARY KEY,
    exam_id UUID NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    question_text TEXT NOT NULL,
    question_type TEXT NOT NULL,
    points FLOAT8 NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE exam_sessions (
    id UUID PRIMARY KEY,
    participant_id UUID NOT NULL,
    exam_id UUID NOT NULL REFERENCES exams(id),
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    status TEXT NOT NULL CHECK (status IN ('in_progress', 'submitted', 'graded'))
);

CREATE INDEX idx_exams_tier ON exams(tier);
CREATE INDEX idx_questions_exam ON questions(exam_id);
CREATE INDEX idx_exam_sessions_participant ON exam_sessions(participant_id);
