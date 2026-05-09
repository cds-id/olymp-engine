# Olympiad LMS Multi-Step with RBAC — Design Document

**Date:** 2026-05-09
**Status:** Approved via brainstorming session

---

## Decisions Made

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| 1 | Crate structure | Granular — 1 crate per domain | PRD has 14 clear modules, easier to test/maintain |
| 2 | Naming | Full rename `blurp-*` → `olymp-*` | Clean break, remove e-commerce remnants |
| 3 | RBAC storage | Hybrid — DB + Redis cache | Multi-dimensional scope too large for JWT, too slow for pure DB |
| 4 | Participant model | Per-stage records (`participant_stages`) | Each stage has own score/rank/status lifecycle |
| 5 | Ranking engine | Async via worker queue | Multi-step calculation, avoid request timeout |
| 6 | Monitoring | SSE + Redis pub/sub | Read-only real-time, simpler than WebSocket |
| 7 | Region hierarchy | Fixed 2-level (province + district) | PRD is explicit 2 levels, YAGNI |

---

## 1. Crate Structure

```
olymp-engine/
├── crates/
│   ├── olymp-core/          # Config, error types, shared types, DB pool, Redis pool
│   ├── olymp-auth/          # JWT, password, OAuth, magic link, user accounts
│   ├── olymp-server/        # main.rs, router, OpenAPI, middleware
│   ├── olymp-notification/  # Email templates, providers (mailgun/smtp)
│   ├── olymp-worker/        # Background job runner (ranking, qualification, cert gen)
│   │
│   ├── olymp-event/         # Event, stage, subject/category master data
│   ├── olymp-region/        # Province, district tables + lookups
│   ├── olymp-rbac/          # Role, permission, scope assignment, cache sync
│   ├── olymp-participant/   # Participant registration, per-stage records, verification
│   ├── olymp-exam/          # Exam, question bank, exam session, answer storage
│   ├── olymp-ranking/       # Score calculation, ranking rules, leaderboard
│   ├── olymp-qualification/ # Promotion logic: stage N → stage N+1
│   ├── olymp-monitoring/    # Audit log, exam progress, cheating log, SSE streaming
│   └── olymp-certificate/   # Certificate/piagam template + generation
```

**13 crates total.**

### Dependency Graph

```
olymp-core ← all crates depend
olymp-auth ← olymp-rbac, olymp-server
olymp-rbac ← olymp-server (middleware), olymp-participant, olymp-exam, olymp-ranking
olymp-event ← olymp-participant, olymp-exam, olymp-ranking, olymp-qualification
olymp-region ← olymp-rbac, olymp-participant
olymp-ranking ← olymp-qualification
olymp-exam ← olymp-monitoring (SSE events)
```

---

## 2. Database Schema — Master Data

### `olymp-event` tables

```sql
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    academic_year TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE education_levels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE subjects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id),
    tier TEXT NOT NULL CHECK (tier IN ('district', 'province', 'national')),
    sequence INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(event_id, tier)
);

CREATE TABLE event_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id),
    education_level_id UUID NOT NULL REFERENCES education_levels(id),
    subject_id UUID NOT NULL REFERENCES subjects(id),
    UNIQUE(event_id, education_level_id, subject_id)
);
```

Stage status values: `draft|open_registration|registration_closed|verification|ready_for_exam|exam_open|exam_closed|scoring|ranking_review|ranking_approved|result_published|promoted|finalized|cancelled`

### `olymp-region` tables

```sql
CREATE TABLE provinces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE districts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    province_id UUID NOT NULL REFERENCES provinces(id),
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    UNIQUE(province_id, slug)
);
```

---

## 3. Database Schema — RBAC

```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT
);

CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE user_role_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id),
    event_id UUID REFERENCES events(id),
    stage_id UUID REFERENCES stages(id),
    province_id UUID REFERENCES provinces(id),
    district_id UUID REFERENCES districts(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ,
    assigned_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE assignment_permission_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES user_role_assignments(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id),
    granted BOOLEAN NOT NULL,
    UNIQUE(assignment_id, permission_id)
);

CREATE INDEX idx_ura_user ON user_role_assignments(user_id);
CREATE INDEX idx_ura_event ON user_role_assignments(event_id);
CREATE INDEX idx_ura_active ON user_role_assignments(is_active, expires_at);
```

### Permission Resolution Flow

1. User login → query `user_role_assignments` WHERE `user_id` AND `is_active` AND not expired
2. For each assignment → get role's `role_permissions`
3. Apply `assignment_permission_overrides` (grant/revoke)
4. Build effective permission set:
```json
{
  "user_id": "...",
  "assignments": [
    {
      "permissions": ["participant.view", "participant.verify", "exam.monitor"],
      "scope": {
        "event_id": "uuid",
        "stage_id": "uuid",
        "province_id": null,
        "district_id": "uuid"
      }
    }
  ]
}
```
5. Cache in Redis key `rbac:{user_id}` with TTL 15 min
6. Invalidate on: role change, assignment change, permission override change

### Middleware Check

```
Request → Extract JWT (user_id) → Read Redis rbac:{user_id}
  → Cache miss? Query DB, rebuild, cache
  → Check: has permission P for resource R within scope S?
  → Allow or 403
```

---

## 4. Database Schema — Participant & Exam

### `olymp-participant`

```sql
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
    participant_id UUID NOT NULL REFERENCES participants(id),
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
```

Participant stage status values: `registered|verified|assigned_to_exam|in_progress|submitted|scored|ranked|qualified|not_qualified|disqualified|winner|finalist`

### `olymp-exam`

```sql
CREATE TABLE exams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_id UUID NOT NULL REFERENCES stages(id),
    event_category_id UUID NOT NULL REFERENCES event_categories(id),
    title TEXT NOT NULL,
    duration_minutes INT NOT NULL,
    total_questions INT NOT NULL DEFAULT 0,
    passing_score FLOAT8,
    status TEXT NOT NULL DEFAULT 'draft',
    opens_at TIMESTAMPTZ,
    closes_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_id UUID NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    question_text TEXT NOT NULL,
    question_type TEXT NOT NULL CHECK (question_type IN ('multiple_choice','essay','true_false')),
    options JSONB,
    correct_answer JSONB,
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
    status TEXT NOT NULL DEFAULT 'assigned',
    is_auto_submitted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(participant_stage_id, exam_id)
);

CREATE TABLE answers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_session_id UUID NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES questions(id),
    answer_data JSONB,
    is_correct BOOLEAN,
    points_earned FLOAT8,
    answered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(exam_session_id, question_id)
);

CREATE INDEX idx_exam_sessions_status ON exam_sessions(status);
CREATE INDEX idx_answers_session ON answers(exam_session_id);
```

### Relationship chain

```
participant → participant_stages (per stage)
                 ↓
            exam_session (per exam in that stage)
                 ↓
              answers (per question)
```

---

## 5. Database Schema — Ranking, Monitoring, Qualification, Certificate

### `olymp-ranking`

```sql
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
    status TEXT NOT NULL DEFAULT 'draft',
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
    qualification_status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ranking_entries_result ON ranking_entries(ranking_result_id);
CREATE INDEX idx_ranking_entries_rank ON ranking_entries(ranking_result_id, rank);
```

### `olymp-monitoring`

```sql
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
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_event ON audit_logs(event_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
```

### `olymp-qualification` — no extra tables

Reads `ranking_entries` WHERE `qualified` → creates `participant_stages` for next stage. Uses:
- `ranking_results` (must be `approved`)
- `ranking_entries` (WHERE `qualification_status = 'qualified'`)
- `stages` (find next by `sequence + 1`)
- Creates new `participant_stages` with `status = 'registered'`

### `olymp-certificate`

```sql
CREATE TABLE certificate_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id),
    name TEXT NOT NULL,
    template_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES certificate_templates(id),
    participant_stage_id UUID NOT NULL REFERENCES participant_stages(id),
    certificate_url TEXT,
    generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_certificates_participant ON certificates(participant_stage_id);
```

---

## 6. API Routes

### RBAC Middleware

```rust
pub struct RbacContext {
    pub user_id: Uuid,
    pub assignments: Vec<EffectiveAssignment>,
}

pub struct EffectiveAssignment {
    pub role: String,
    pub permissions: HashSet<String>,
    pub event_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
}

impl RbacContext {
    pub fn can(&self, permission: &str, scope: &ResourceScope) -> bool;
    pub fn accessible_events(&self) -> Vec<Uuid>;
    pub fn accessible_stages(&self, event_id: Uuid) -> Vec<Uuid>;
}
```

### Route Map

```
# Auth
POST   /api/auth/login
POST   /api/auth/register
POST   /api/auth/refresh
POST   /api/auth/logout
POST   /api/auth/magic-link
POST   /api/auth/oauth/{provider}

# Master Data
GET    /api/events
POST   /api/events                          # olympiad.master.create
GET    /api/events/{event_id}
PUT    /api/events/{event_id}               # olympiad.master.update
GET    /api/events/{event_id}/stages
POST   /api/events/{event_id}/stages        # olympiad.stage.manage
PUT    /api/stages/{stage_id}               # olympiad.stage.manage
GET    /api/education-levels
GET    /api/subjects
GET    /api/provinces
GET    /api/provinces/{province_id}/districts

# RBAC
GET    /api/rbac/roles                      # rbac.role.view
POST   /api/rbac/roles                      # rbac.role.create
PUT    /api/rbac/roles/{role_id}            # rbac.role.update
GET    /api/rbac/permissions                # rbac.role.view
POST   /api/rbac/roles/{role_id}/permissions  # rbac.permission.assign
GET    /api/rbac/assignments                # rbac.user.assign
POST   /api/rbac/assignments                # rbac.user.assign
PUT    /api/rbac/assignments/{id}           # rbac.user.assign
DELETE /api/rbac/assignments/{id}           # rbac.user.assign
GET    /api/rbac/audit                      # rbac.audit.view

# Participants
GET    /api/events/{event_id}/participants                  # participant.view
POST   /api/events/{event_id}/participants                  # participant.create
POST   /api/events/{event_id}/participants/import           # participant.import
GET    /api/participants/{id}                               # participant.view
PUT    /api/participants/{id}                               # participant.update
POST   /api/participants/{id}/verify                        # participant.verify
POST   /api/participants/{id}/approve                       # participant.approve
POST   /api/participants/{id}/reject                        # participant.reject
GET    /api/stages/{stage_id}/participants                  # participant.view (scoped)
POST   /api/events/{event_id}/participants/export           # participant.export

# Exams
GET    /api/stages/{stage_id}/exams                         # exam.view
POST   /api/stages/{stage_id}/exams                         # exam.create
PUT    /api/exams/{exam_id}                                 # exam.update
POST   /api/exams/{exam_id}/open                            # exam.start
POST   /api/exams/{exam_id}/close                           # exam.close
POST   /api/exams/{exam_id}/assign                          # exam.assign
GET    /api/exams/{exam_id}/sessions                        # exam.monitor
POST   /api/exam-sessions/{id}/force-submit                 # exam.force_submit
POST   /api/exam-sessions/{id}/reset                        # exam.reset_attempt
GET    /api/exam-sessions/{id}/logs                         # exam.log_view

# Exam Taking (peserta)
GET    /api/my/exam-sessions
GET    /api/my/exam-sessions/{id}
POST   /api/my/exam-sessions/{id}/start
PUT    /api/my/exam-sessions/{id}/answers/{question_id}
POST   /api/my/exam-sessions/{id}/submit

# Monitoring (SSE)
GET    /api/exams/{exam_id}/monitor/stream                  # SSE
GET    /api/exam-sessions/{id}/progress                     # polling fallback

# Ranking
GET    /api/stages/{stage_id}/ranking                       # ranking.view
POST   /api/stages/{stage_id}/ranking/calculate             # ranking.calculate (async)
GET    /api/stages/{stage_id}/ranking/status                 # job status
POST   /api/stages/{stage_id}/ranking/review                # ranking.review
POST   /api/stages/{stage_id}/ranking/approve               # ranking.approve
POST   /api/stages/{stage_id}/ranking/publish               # ranking.publish
POST   /api/events/{event_id}/participants/export           # ranking.export

# Qualification
POST   /api/stages/{stage_id}/promote                       # qualification.process (async)

# Certificates
POST   /api/events/{event_id}/certificates/templates
POST   /api/stages/{stage_id}/certificates/generate         # async
GET    /api/my/certificates
```

---

## 7. Phased Implementation Plan

### Phase 1 — Foundation (rename + core + region + event)

| Task | Crate | Description |
|------|-------|-------------|
| 1.1 | all | Rename `blurp-*` → `olymp-*`, clean e-commerce remnants from config |
| 1.2 | `olymp-core` | Clean types: remove Money/Currency/Address, add shared olympiad types |
| 1.3 | `olymp-region` | New crate: provinces + districts tables, seed data, CRUD handlers |
| 1.4 | `olymp-event` | New crate: events, stages, education_levels, subjects, event_categories |
| 1.5 | `olymp-server` | Update router, remove e-commerce routes, wire event + region routes |
| 1.6 | — | Verify: `cargo build`, migrations run, Swagger UI shows new endpoints |

### Phase 2 — RBAC

| Task | Crate | Description |
|------|-------|-------------|
| 2.1 | `olymp-auth` | Remove `is_admin` flag, add `user_role_assignments` FK |
| 2.2 | `olymp-rbac` | New crate: roles, permissions, role_permissions, assignments, overrides |
| 2.3 | `olymp-rbac` | Permission seeder — insert all permissions from PRD Section 5 |
| 2.4 | `olymp-rbac` | Role template seeder — Superadmin, Admin, Panitia with default permissions |
| 2.5 | `olymp-rbac` | Redis cache: build effective permission set, cache/invalidate logic |
| 2.6 | `olymp-rbac` | RBAC middleware: extract JWT → resolve permissions → inject RbacContext |
| 2.7 | `olymp-rbac` | Assignment CRUD handlers + audit logging |
| 2.8 | `olymp-server` | Wire RBAC middleware into router |
| 2.9 | — | Verify: Panitia scoped to district cannot access other district data |

### Phase 3 — Participant

| Task | Crate | Description |
|------|-------|-------------|
| 3.1 | `olymp-participant` | Rewrite models: participants + participant_stages per new schema |
| 3.2 | `olymp-participant` | Registration handler: create participant + initial participant_stage |
| 3.3 | `olymp-participant` | Verify/approve/reject handlers with RBAC scope check |
| 3.4 | `olymp-participant` | Import endpoint (CSV/Excel) |
| 3.5 | `olymp-participant` | Export endpoint |
| 3.6 | `olymp-participant` | Duplicate detection: same user + event + subject |
| 3.7 | — | Verify: participant scoped to event+stage+region, panitia can only see their scope |

### Phase 4 — Exam

| Task | Crate | Description |
|------|-------|-------------|
| 4.1 | `olymp-exam` | Rewrite models per new schema |
| 4.2 | `olymp-exam` | Exam CRUD + question bank management |
| 4.3 | `olymp-exam` | Assign participants to exam sessions |
| 4.4 | `olymp-exam` | Exam taking flow: start → save answers → submit |
| 4.5 | `olymp-exam` | Auto-submit on timeout (worker job or scheduler) |
| 4.6 | `olymp-exam` | Force submit + reset attempt (admin) |
| 4.7 | `olymp-exam` | Open/close exam status transitions with validation |
| 4.8 | — | Verify: peserta can take exam, answers persisted, auto-submit works |

### Phase 5 — Monitoring

| Task | Crate | Description |
|------|-------|-------------|
| 5.1 | `olymp-monitoring` | Rewrite: cheating_logs, exam_progress, audit_logs per new schema |
| 5.2 | `olymp-monitoring` | Cheating log ingestion endpoint |
| 5.3 | `olymp-monitoring` | Exam progress tracker: update on each answer save |
| 5.4 | `olymp-monitoring` | SSE endpoint: Redis pub/sub → stream to panitia |
| 5.5 | `olymp-monitoring` | Audit log middleware: auto-record CUD operations |
| 5.6 | — | Verify: SSE stream shows live progress, cheating logs recorded |

### Phase 6 — Ranking + Qualification

| Task | Crate | Description |
|------|-------|-------------|
| 6.1 | `olymp-ranking` | Rewrite: ranking_rules, ranking_results, ranking_entries |
| 6.2 | `olymp-ranking` | Ranking calculation job (worker) |
| 6.3 | `olymp-ranking` | Review → approve → publish flow with RBAC |
| 6.4 | `olymp-qualification` | New crate: promotion logic |
| 6.5 | `olymp-qualification` | Validation: ranking approved, next stage exists |
| 6.6 | `olymp-worker` | Wire ranking + qualification + auto-submit jobs |
| 6.7 | — | Verify: Daerah → Provinsi → Nasional promotion end-to-end |

### Phase 7 — Certificate + Finalization

| Task | Crate | Description |
|------|-------|-------------|
| 7.1 | `olymp-certificate` | New crate: certificate_templates, certificates tables |
| 7.2 | `olymp-certificate` | Template upload (S3) |
| 7.3 | `olymp-certificate` | PDF generation job (worker) |
| 7.4 | `olymp-certificate` | Peserta download endpoint |
| 7.5 | `olymp-event` | Event finalization flow |
| 7.6 | — | Verify: certificates generated, event finalized |

---

**7 phases. Each independently deployable & testable.**
- Phase 1-2: Foundation (~week 1-2)
- Phase 3-4: Core business (~week 3-4)
- Phase 5-6: Advanced features (~week 5-6)
- Phase 7: Polish (~week 7)
