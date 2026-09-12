.PHONY: help dev rpc-gen rpc-sync-check up down logs test smoke \
	up-mysql up-sqlite down-mysql down-sqlite smoke-mysql smoke-sqlite \
	up-dev-auth down-dev-auth up-with-dev-auth down-with-dev-auth test-e2e-stack \
	db-migrate db-switch-dialect db-matrix

COMPOSE ?= docker compose
COMPOSE_FILE ?= docker-compose.yml

help:
	@echo "Octanest targets:"
	@echo "  make dev            - local API + web (Vite proxy; D-10)"
	@echo "  make rpc-gen        - regenerate packages/api-client from Rust"
	@echo "  make rpc-sync-check - fail if generated client is out of sync"
	@echo "  make up             - docker compose up (Traefik on :80)"
	@echo "  make up-mysql       - compose up with the MySQL profile (D-07)"
	@echo "  make up-sqlite      - compose up with SQLite file in ./var (D-19)"
	@echo "  make up-dev-auth    - Mailpit + OIDC mock + Resend/WorkOS stubs (docs/dev-auth.md)"
	@echo "  make up-with-dev-auth - make up + attach API to stubs (SMTP→Mailpit)"
	@echo "  make test-e2e-stack - full Vitest e2e vs API + Mailpit/OIDC/stubs"
	@echo "  make down           - docker compose down"
	@echo "  make down-mysql     - docker compose down (mysql overlay)"
	@echo "  make down-sqlite    - docker compose down (sqlite overlay)"
	@echo "  make down-dev-auth  - stop local auth/email stubs"
	@echo "  make down-with-dev-auth - tear down full stack from up-with-dev-auth"
	@echo "  make logs           - follow compose logs"
	@echo "  make test           - cargo nextest + JS Vitest (unit/integration/e2e)"
	@echo "  make smoke          - compose bring-up smoke (PLAT-01)"
	@echo "  make smoke-mysql    - bring-up smoke asserting dialect=mysql"
	@echo "  make smoke-sqlite   - bring-up smoke asserting dialect=sqlite"
	@echo "  make db-migrate     - apply migrations for DATABASE_URL"
	@echo "  make db-switch-dialect - migrate an EMPTY target DB to a new dialect"
	@echo "  make db-matrix      - run the dialect probe test against DATABASE_URL"
	@echo ""
	@echo "Sample DATABASE_URLs:"
	@echo "  postgres://octanest:octanest@localhost:5432/octanest"
	@echo "  mysql://octanest:octanest@127.0.0.1:3306/octanest"
	@echo "  sqlite:./var/octanest.db"

dev:
	@echo "Starting API + web (rpc-gen once)..."
	@$(MAKE) rpc-gen
	@echo "Run in two terminals:"
	@echo "  OCTANEST_ENV=development API_BIND=127.0.0.1:8080 cargo run -p octanest-api --bin octanest-api"
	@echo "  bun run --filter @octanest/web dev"

rpc-gen:
	cargo run -q -p octanest-api --bin rpc-gen

rpc-sync-check:
	@./scripts/check-rpc-sync.sh

up:
	$(COMPOSE) -f $(COMPOSE_FILE) up --build -d

up-mysql:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.mysql.yml --profile mysql up --build -d

up-sqlite:
	mkdir -p var
	@host="$$(./scripts/sqlite-host-dir.sh)"; \
	printf 'OCTANEST_SQLITE_HOST_DIR=%s\n' "$$host" > .env.sqlite; \
	$(COMPOSE) --env-file .env.sqlite -f docker-compose.yml -f docker-compose.sqlite.yml up --build -d

down:
	$(COMPOSE) -f $(COMPOSE_FILE) down --remove-orphans

down-mysql:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.mysql.yml down --remove-orphans

down-sqlite:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.sqlite.yml down --remove-orphans

up-dev-auth:
	@bash -c 'source ./scripts/docker-wsl-creds.sh; $(COMPOSE) -f docker-compose.dev-auth.yml --profile dev-auth up --build -d'
	@echo "==> Mailpit UI  http://127.0.0.1:8025"
	@echo "==> OIDC mock   http://127.0.0.1:9090/default"
	@echo "==> HTTP stubs  http://127.0.0.1:9092"
	@echo "==> Env template: cp docs/dev-auth.env.example .env.dev-auth"
	@echo "==> Compose API: make up-with-dev-auth (SMTP→Mailpit)"
	@echo "==> Docs: docs/dev-auth.md"

# Main Traefik stack + Mailpit/stubs; API SMTP defaults to smtp://mailpit:1025.
up-with-dev-auth:
	@bash -c 'source ./scripts/docker-wsl-creds.sh; \
	  $(COMPOSE) -f docker-compose.yml \
	    -f docker-compose.dev-auth.yml \
	    -f docker-compose.dev-auth-attach.yml \
	    --profile dev-auth up --build -d'
	@./scripts/dev-auth/promote-smtp-settings.sh
	@echo "==> App        http://localhost"
	@echo "==> Mailpit UI http://127.0.0.1:8025"
	@echo "==> OIDC mock  http://127.0.0.1:9090/default"
	@echo "==> HTTP stubs http://127.0.0.1:9092"
	@echo "==> Optional env: docs/dev-auth.env.compose.example"
	@echo "==> Docs: docs/dev-auth.md"

down-dev-auth:
	@bash -c 'source ./scripts/docker-wsl-creds.sh; $(COMPOSE) -f docker-compose.dev-auth.yml --profile dev-auth down --remove-orphans'

down-with-dev-auth:
	@bash -c 'source ./scripts/docker-wsl-creds.sh; \
	  $(COMPOSE) -f docker-compose.yml \
	    -f docker-compose.dev-auth.yml \
	    -f docker-compose.dev-auth-attach.yml \
	    --profile dev-auth down --remove-orphans'

test-e2e-stack:
	./scripts/dev-auth/run-stack-e2e.sh

logs:
	$(COMPOSE) -f $(COMPOSE_FILE) logs -f

test:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --workspace; \
	else \
		echo "cargo-nextest not found; falling back to cargo test (install: cargo install cargo-nextest --locked)"; \
		cargo test --workspace; \
	fi
	bun run test

smoke:
	@EXPECT_DIALECT=postgres ./scripts/compose-smoke.sh

smoke-mysql:
	@COMPOSE_FILES="-f docker-compose.yml -f docker-compose.mysql.yml" COMPOSE_PROFILES=mysql EXPECT_DIALECT=mysql ./scripts/compose-smoke.sh

smoke-sqlite:
	mkdir -p var
	@host="$$(./scripts/sqlite-host-dir.sh)"; \
	OCTANEST_SQLITE_HOST_DIR="$$host" COMPOSE_FILES="-f docker-compose.yml -f docker-compose.sqlite.yml" EXPECT_DIALECT=sqlite ./scripts/compose-smoke.sh; \
	if echo "$$host" | grep -Eq '^[A-Za-z]:/'; then \
	  src="$$(wslpath "$$host")/octanest.db"; \
	  if [ -f "$$src" ]; then cp -f "$$src" ./var/octanest.db; echo "==> mirrored $$src -> ./var/octanest.db"; fi; \
	fi

db-migrate:
	cargo run -q -p octanest-db --bin migrate

db-switch-dialect:
	./scripts/db-switch-dialect.sh $(ARGS)

db-matrix:
	cargo test -p octanest-db --test dialect_probe -- --nocapture
