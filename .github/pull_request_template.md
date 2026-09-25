## Summary

<!-- What this changes and why. Link issues (e.g. "Fixes #123") or the roadmap phase it belongs to. -->

## Changes

-
-

## Test plan

<!-- Check what you actually ran. CI runs the full matrix: Rust nextest, web lint/format, Vitest, stack e2e, RPC sync, Compose. -->

- [ ] `make test` (Rust nextest + Vitest)
- [ ] `make web-lint` and `make web-format-check` — required when `apps/web` changes
- [ ] `make rpc-gen` run and regenerated `packages/api-client` committed — required when RPC procedures or shared types change
- [ ] `make rpc-sync-check`
- [ ] `make test-e2e-stack` — for auth, UI, or cross-stack behavior
- [ ] `docker compose config` / `make up && make smoke` — for Compose or deploy changes
- [ ] Not run — explain below

## Checklist

- [ ] Behavior changes include tests (see [docs/TESTING.md](docs/TESTING.md))
- [ ] No secrets committed (`.env`, tokens, keys); examples go in `.env.example` or `docs/*.example`
- [ ] New env vars documented in `.env.example` and [docs/CONFIGURATION.md](docs/CONFIGURATION.md)
- [ ] UI follows Octane `.tsrx` conventions (see [AGENTS.md](AGENTS.md))
- [ ] Docs updated where behavior, config, or commands changed
