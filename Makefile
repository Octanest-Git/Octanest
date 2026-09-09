.PHONY: help dev rpc-gen rpc-sync-check up down logs test smoke \
	up-mysql up-sqlite down-mysql down-sqlite smoke-mysql smoke-sqlite \
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
	@echo "  make down           - docker compose down"
	@echo "  make down-mysql     - docker compose down (mysql overlay)"
	@echo "  make down-sqlite    - docker compose down (sqlite overlay)"
	@echo "  make logs           - follow compose logs"
	@echo "  make test           - cargo + JS tests"
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

logs:
	$(COMPOSE) -f $(COMPOSE_FILE) logs -f

test:
	cargo test --workspace
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
