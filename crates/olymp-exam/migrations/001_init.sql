-- olymp-exam: exams, questions, exam_sessions, answers

CREATE TABLE exams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_id UUID NOT NULL REFERENCES stages(id),
    title TEXT NOT NULL,
    description TEXT,
    duration_minutes INT NOT NULL,
    max_attempts INT NOT NULL DEFAULT 1,
    shuffle_questions BOOLEAN NOT NULL DEFAULT false,
    shuffle_options BOOLEAN NOT NULL DEFAULT false,
    opens_at TIMESTAMPTZ,
    closes_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_id UUID NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    question_text TEXT NOT NULL,
    question_type TEXT NOT NULL CHECK (question_type IN ('multiple_choice', 'essay', 'true_false')),
    options JSONB,           -- for MCQ: [{"key":"A","text":"..."},{"key":"B","text":"..."}]
    correct_answer JSONB,    -- for MCQ: "A" or ["A","C"]; for true_false: true/false
    points FLOAT8 NOT NULL,
    sequence INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE exam_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    participant_stage_id UUID NOT NULL REFERENCES participant_stages(id),
    exam_id UUID NOT NULL REFERENCES exams(id),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    score FLOAT8,
    completion_time_secs INT,
    status TEXT NOT NULL DEFAULT 'assigned',
    is_auto_submitted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(participant_stage_id, exam_id)
);

CREATE TABLE answers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_session_id UUID NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES questions(id),
    answer_data JSONB,       -- MCQ: "A"; essay: "long text..."; true_false: true/false
    is_correct BOOLEAN,
    points_earned FLOAT8,
    answered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(exam_session_id, question_id)
);

CREATE INDEX idx_exams_stage ON exams(stage_id);
CREATE INDEX idx_questions_exam ON questions(exam_id);
CREATE INDEX idx_questions_sequence ON questions(exam_id, sequence);
CREATE INDEX idx_exam_sessions_status ON exam_sessions(status);
CREATE INDEX idx_exam_sessions_ps ON exam_sessions(participant_stage_id);
CREATE INDEX idx_answers_session ON answers(exam_session_id);
