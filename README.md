# Olympiad LMS Engine

Rust backend for multi-tier Olympiad Learning Management System. Manages participant registration, exam execution, monitoring, ranking, and tier progression (District → Province → National).

## Architecture

Cargo workspace with 9 crates:

- **olymp-core** — Shared types, config, error handling, response envelope
- **olymp-auth** — JWT + Redis revocation, user authentication
- **olymp-notification** — Email/SMS notifications
- **olymp-worker** — Background jobs (Redis ZSET delayed queue)
- **olymp-server** — HTTP server, route composition
- **olymp-participant** — Registration, tier management, participant profiles
- **olymp-exam** — Exam creation, question management, answer submission
- **olymp-ranking** — Scoring, leaderboard, tier advancement
- **olymp-monitoring** — Progress tracking, exam monitoring, analytics

## Tech Stack

- **Framework:** Axum 0.8
- **Database:** PostgreSQL 16 (per-crate schemas, immutable logs)
- **Cache:** Redis 7 (AOF persistence)
- **Auth:** JWT + Redis token revocation
- **Logging:** PostgreSQL audit logs + Redis AOF (compliance-compliant, immutable)

## Quick Start

```bash
# Start PostgreSQL + Redis (with immutable logging)
docker-compose up -d

# Run migrations
make migrate

# Start server
cargo run -p olymp-server
```

Server runs on `http://localhost:8080`

## Core Features

### Participant Management
- Multi-tier registration (District, Province, National)
- Participant profiles with tier assignment
- Automatic tier progression based on ranking
- Participant locking per tier (approval indicator)

### Exam Execution
- Question bank management
- Timed exam sessions
- Real-time answer submission
- Exam state tracking

### Ranking & Advancement
- Automated scoring
- Leaderboard generation
- Tier-based advancement rules
- Ranking history tracking

### Monitoring
- Live exam progress tracking
- Participant analytics
- Exam statistics
- Audit trail (immutable logs)

## API Endpoints

All responses use standard envelope: `{ "data": ..., "error": null/obj, "meta": null/obj }`

### Auth
- `POST /api/auth/register` — Register participant
- `POST /api/auth/login` — Login, returns JWT
- `POST /api/auth/logout` — Revoke token
- `POST /api/auth/refresh` — Refresh access token

### Participant
- `POST /api/participant/register` — Register for tier
- `GET /api/participant/profile` — Get participant profile
- `GET /api/participant/tier` — Get current tier info
- `GET /api/participant/history` — Get tier progression history

### Exam
- `GET /api/exam/list` — List available exams
- `GET /api/exam/{id}` — Exam details
- `POST /api/exam/{id}/start` — Start exam session
- `POST /api/exam/{id}/submit-answer` — Submit answer
- `POST /api/exam/{id}/finish` — Complete exam

### Ranking
- `GET /api/ranking/leaderboard` — Get tier leaderboard
- `GET /api/ranking/my-rank` — Get participant rank
- `GET /api/ranking/advancement` — Check advancement eligibility
- `POST /api/ranking/advance-tier` — Advance to next tier

### Monitoring
- `GET /api/monitoring/exam-progress` — Live exam progress
- `GET /api/monitoring/analytics` — Exam analytics
- `GET /api/monitoring/audit-log` — Immutable audit trail

## Database Schema

Per-crate migrations:
- `olymp-auth/migrations` — User authentication
- `olymp-participant/migrations` — Participant data
- `olymp-exam/migrations` — Exam & questions
- `olymp-ranking/migrations` — Scores & rankings
- `olymp-monitoring/migrations` — Audit logs

## Logging & Compliance

- PostgreSQL logs all statements (immutable, retained per compliance)
- Redis AOF persistence (append-only file)
- Audit trail in `olymp-monitoring` schema
- All logs timestamped and tamper-evident

## Configuration

Environment variables (or `.env.local`):

```bash
DATABASE_URL=postgres://olymp:olymp@localhost:5432/olymp
REDIS_URL=redis://localhost:6379

OLYMP__AUTH__JWT_SECRET=your-secret
OLYMP__TIER__ADVANCEMENT_THRESHOLD=75
OLYMP__EXAM__SESSION_TIMEOUT=120
```

## Testing

```bash
# Unit tests
cargo test

# E2E tests (server must be running)
cargo run -p olymp-server &
cargo test -p olymp-server -- --test-threads=1
```

## License

MIT — CDS Indonesia
