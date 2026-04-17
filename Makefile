.PHONY: dev setup db backend frontend docker clean test

# ── Quick Start ──────────────────────────────────────
setup: ## First-time setup
	cp -n .env.example .env || true
	cd frontend && npm install --legacy-peer-deps
	@echo ""
	@echo "✓ Setup complete. Edit .env with your API keys, then run: make dev"

dev: ## Run everything in dev mode (requires Docker for DB)
	make db &
	sleep 3
	make backend &
	make frontend

# ── Individual Services ──────────────────────────────
db: ## Start PostgreSQL with pgvector
	docker run --rm --name nodum-db \
		-e POSTGRES_DB=nodum \
		-e POSTGRES_USER=nodum \
		-e POSTGRES_PASSWORD=nodum_dev_password \
		-p 5432:5432 \
		pgvector/pgvector:pg16

backend: ## Run Rust backend
	cd backend && cargo run

frontend: ## Run React dev server
	cd frontend && npm run dev

# ── Docker ───────────────────────────────────────────
docker: ## Build and run with Docker Compose
	docker compose up --build

docker-down: ## Stop Docker Compose
	docker compose down

# ── Build ────────────────────────────────────────────
build-backend: ## Build Rust binary (release)
	cd backend && cargo build --release

build-frontend: ## Build frontend for production
	cd frontend && npm run build

build: build-backend build-frontend ## Build everything

# ── Database ─────────────────────────────────────────
db-reset: ## Reset database (WARNING: destroys data)
	docker exec -i nodum-db psql -U nodum -d nodum -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	@echo "Database reset. Restart the backend to re-run migrations."

db-shell: ## Open psql shell
	docker exec -it nodum-db psql -U nodum -d nodum

# ── Testing ──────────────────────────────────────────
test: ## Run backend tests
	cd backend && cargo test

lint: ## Lint everything
	cd backend && cargo clippy
	cd frontend && npm run lint

# ── Cleanup ──────────────────────────────────────────
clean: ## Remove build artifacts
	cd backend && cargo clean
	rm -rf frontend/node_modules frontend/dist
	docker compose down -v 2>/dev/null || true

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
