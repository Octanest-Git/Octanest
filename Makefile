.PHONY: help dev rpc-gen rpc-sync-check up down logs test smoke

COMPOSE ?= docker compose
COMPOSE_FILE ?= docker-compose.yml

help:
	@echo "Octanest targets:"
	@echo "  make dev            - local API + web (Vite proxy; D-10)"
	@echo "  make rpc-gen        - regenerate packages/api-client from Rust"
	@echo "  make rpc-sync-check - fail if generated client is out of sync"
	@echo "  make up             - docker compose up (Traefik on :80)"
	@echo "  make down           - docker compose down"
	@echo "  make logs           - follow compose logs"
	@echo "  make test           - cargo + JS tests"
	@echo "  make smoke          - compose bring-up smoke (PLAT-01)"

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

down:
	$(COMPOSE) -f $(COMPOSE_FILE) down --remove-orphans

logs:
	$(COMPOSE) -f $(COMPOSE_FILE) logs -f

test:
	cargo test --workspace
	bun run test

smoke:
	@./scripts/compose-smoke.sh
