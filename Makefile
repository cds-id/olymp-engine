.PHONY: db-up db-down db-reset db-logs db-shell migrate help

help:
	@echo "Blurp Engine Development Commands"
	@echo "=================================="
	@echo "make db-up       - Start Postgres + Redis containers"
	@echo "make db-down     - Stop containers (keep volumes)"
	@echo "make db-reset    - Stop, remove volumes, restart (DESTRUCTIVE)"
	@echo "make db-logs     - Tail container logs"
	@echo "make db-shell    - Connect to Postgres psql"
	@echo "make migrate     - Run all database migrations"

db-up:
	docker-compose up -d
	@echo "✓ Postgres on localhost:5432 (blurp/blurp)"
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

db-logs:
	docker-compose logs -f

db-shell:
	docker-compose exec postgres psql -U blurp -d blurp

migrate:
	@echo "Running migrations..."
	@for crate in auth catalog cart payment media orders admin; do \
		echo "Migrating $$crate..."; \
		sqlx migrate run --database-url "postgres://blurp:blurp@localhost:5432/blurp" -D "crates/blurp-$$crate/migrations" || true; \
	done
	@echo "✓ All migrations complete"

check:
	cargo check --workspace

build:
	cargo build --release

run:
	cargo run --bin blurp-server

test:
	cargo test --workspace

clean:
	cargo clean
	docker-compose down -v
