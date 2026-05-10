#!/usr/bin/env bash
set -euo pipefail

# ─── Olymp Engine Migration Runner ───
# Runs all crate migrations in dependency order against Postgres.
# Idempotent: safe to run multiple times.
# Uses docker exec to run psql inside the olymp-postgres container.
#
# Usage:
#   ./scripts/migrate.sh                              # default container + creds
#   CONTAINER=my-pg PGUSER=foo PGDB=bar ./scripts/migrate.sh

CONTAINER="${CONTAINER:-olymp-postgres}"
PGUSER="${PGUSER:-olymp}"
PGDB="${PGDB:-olymp}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Migration order matters — foreign keys require parent tables first
CRATE_ORDER=(
    "olymp-auth"         # auth.users (referenced by many)
    "olymp-region"       # provinces, districts
    "olymp-rbac"         # roles, permissions, assignments (refs auth.users)
    "olymp-event"        # events, stages, education_levels, subjects
    "olymp-participant"  # participants, participant_stages (refs auth.users, events, stages)
    "olymp-exam"         # exams, questions, sessions, answers (refs stages, participant_stages)
    "olymp-monitoring"   # cheating_logs, exam_progress, audit_logs
    "olymp-ranking"      # ranking_rules, ranking_results, ranking_entries
    "olymp-certificate"  # certificate_templates, certificates
)

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔══════════════════════════════════════╗"
echo "║   Olymp Engine Migration Runner      ║"
echo "╚══════════════════════════════════════╝"
echo ""
echo "Container: $CONTAINER | DB: $PGDB | User: $PGUSER"
echo ""

# Check container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER}$"; then
    echo -e "${RED}✗ Container '$CONTAINER' not running.${NC}"
    echo "  Try: make db-up"
    exit 1
fi

# Check DB is reachable
if ! docker exec "$CONTAINER" psql -U "$PGUSER" -d "$PGDB" -c "SELECT 1" &> /dev/null; then
    echo -e "${RED}✗ Cannot connect to database inside container.${NC}"
    exit 1
fi

run_sql_file() {
    local sqlfile="$1"
    # Pipe SQL into psql inside the container
    docker exec -i "$CONTAINER" psql -U "$PGUSER" -d "$PGDB" -v ON_ERROR_STOP=0 < "$sqlfile" 2>&1
}

TOTAL=0
APPLIED=0
SKIPPED=0
FAILED=0

for crate in "${CRATE_ORDER[@]}"; do
    MIGRATION_DIR="$PROJECT_ROOT/crates/$crate/migrations"

    if [ ! -d "$MIGRATION_DIR" ]; then
        echo -e "${YELLOW}⊘ $crate — no migrations dir${NC}"
        continue
    fi

    # Get sorted SQL files
    mapfile -t FILES < <(find "$MIGRATION_DIR" -name "*.sql" -type f | sort)

    if [ ${#FILES[@]} -eq 0 ]; then
        echo -e "${YELLOW}⊘ $crate — no .sql files${NC}"
        continue
    fi

    echo -e "─── ${GREEN}$crate${NC} (${#FILES[@]} files) ───"

    for sqlfile in "${FILES[@]}"; do
        filename=$(basename "$sqlfile")
        TOTAL=$((TOTAL + 1))

        output=$(run_sql_file "$sqlfile" 2>&1) || true

        if echo "$output" | grep -qiE "^ERROR:"; then
            # Check if benign
            if echo "$output" | grep -qiE "already exists|duplicate key|does not exist"; then
                echo -e "  ${YELLOW}⊘ $filename (already applied)${NC}"
                SKIPPED=$((SKIPPED + 1))
            else
                echo -e "  ${RED}✗ $filename${NC}"
                echo "$output" | grep -i "^ERROR:" | head -3 | sed 's/^/    /'
                FAILED=$((FAILED + 1))
            fi
        else
            echo -e "  ${GREEN}✓ $filename${NC}"
            APPLIED=$((APPLIED + 1))
        fi
    done
done

echo ""
echo "════════════════════════════════════════"
echo -e "Total: $TOTAL | ${GREEN}Applied: $APPLIED${NC} | ${YELLOW}Skipped: $SKIPPED${NC} | ${RED}Failed: $FAILED${NC}"

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}⚠  Some migrations failed!${NC}"
    exit 1
else
    echo -e "${GREEN}✓ All migrations complete${NC}"
fi
