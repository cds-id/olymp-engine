.PHONY: help db-up db-down db-reset db-logs db-shell migrate check build run test clean

help:
	@echo "Olymp Engine Development Commands"
	@echo "=================================="
	@echo ""
	@echo "Database:"
	@echo "  make db-up       - Start Postgres + Redis containers"
	@echo "  make db-down     - Stop containers (keep volumes)"
	@echo "  make db-reset    - Stop, remove volumes, restart (DESTRUCTIVE)"
	@echo "  make db-logs     - Tail container logs"
	@echo "  make db-shell    - Connect to Postgres psql"
	@echo ""
	@echo "Migrations:"
	@echo "  make migrate     - Run all database migrations (idempotent)"
	@echo ""
	@echo "Build & Run:"
	@echo "  make check       - Cargo check workspace"
	@echo "  make build       - Cargo build release"
	@echo "  make run         - Run olymp-server (dev)"
	@echo "  make test        - Run all workspace tests"
	@echo "  make clean       - Cargo clean + remove containers/volumes"

db-up:
	docker-compose up -d
	@echo "✓ Postgres on localhost:5432 (olymp/olymp)"
	@echo "✓ Redis on localhost:6379"
	@sleep 3
	@docker-compose ps

db-down:
	docker-compose down

db-reset:
	@echo "⚠️  Dropping all data..."
	docker-compose down -v
	docker-compose up -d
	@sleep 3
	@echo "✓ Fresh containers started"
	@echo "Run 'make migrate' to re-create schema"

db-logs:
	docker-compose logs -f

db-shell:
	docker-compose exec postgres psql -U olymp -d olymp

migrate:
	@./scripts/migrate.sh

check:
	cargo check --workspace

build:
	cargo build --release

run:
	cargo run --bin olymp-server

test:
	cargo test --workspace -- --test-threads=1

clean:
	cargo clean
	docker-compose down -v
