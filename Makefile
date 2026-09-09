.PHONY: help dev rpc-gen rpc-sync-check up down test smoke

help:
	@echo "Octanest targets:"
	@echo "  make dev            - local API + web (filled in later plans)"
	@echo "  make rpc-gen        - regenerate packages/api-client from Rust"
	@echo "  make rpc-sync-check - fail if generated client is out of sync"
	@echo "  make up             - docker compose up"
	@echo "  make down           - docker compose down"
	@echo "  make test           - cargo + JS tests"
	@echo "  make smoke          - compose smoke script (plan 01-04)"

dev:
	@echo "TODO: make dev — wire API + web + rpc watch in plans 01-02/01-03"

rpc-gen:
	cargo run -q -p octanest-api --bin rpc-gen

rpc-sync-check:
	@./scripts/check-rpc-sync.sh

up:
	@echo "TODO: make up — docker compose in plan 01-04"

down:
	@echo "TODO: make down — docker compose in plan 01-04"

test:
	cargo test --workspace
	bun run test

smoke:
	@echo "TODO: make smoke — plan 01-04"
