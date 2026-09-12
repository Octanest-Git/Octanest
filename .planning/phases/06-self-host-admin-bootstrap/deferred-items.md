# Deferred items — Phase 06

## Resolved in 06-03

| Item | Resolved | Notes |
|------|----------|-------|
| `bootstrap_strict_rpc_allowlist_while_needs_setup` | 06-03 | D-11 allowlist in `rpc::dispatch` |
| `bootstrap_allow_signup_*` stubs | 06-03 | Wizard persists `allow_signup`; signup_closed enforced |

No open deferred items from 06-02/06-03.

## Post-close addendum (2026-09-12)

Shipped after Phase 06 verification closed — not a new phase plan. Phase 7+ agents should treat these as **current codebase truth** (do not re-discover or regress).

| Area | What landed | Key paths |
|------|-------------|-----------|
| Setup auth stack | Wizard selects local / WorkOS / OIDC; persists public provider fields on bootstrap | `setup.index.tsrx`, `BootstrapSetupRequest`, `bootstrap_setup` |
| Factory reset | Sys-admin RPC + Admin Auth danger zone (`RESET`) → wipe → `needs_setup` | `admin.instance.factory_reset`, `admin/auth.tsrx`, `Database::factory_reset_instance` |
| Auth SSR / no form skeleton | `/login`, `/signup`, `/setup` loaders; forms `method="post" action="#"` + button handlers | `ssr-auth.ts`, auth routes |
| OIDC harden | Reqwest connect 5s / request 10s; mock healthcheck + compose issuer docs | `auth/oidc.rs`, `docker-compose.dev-auth.yml`, `docs/dev-auth.md` |
| SSO query alias | `returnTo` accepted alongside `return_to` on SSO start | `auth_callbacks.rs` |
| TanStack Query | Root `QueryClientProvider`; shared soft `auth.me` + bootstrap + providerConfig; `/status` + admin settings via Query; cache clear/update on logout/profile/factory reset | `lib/query-client.ts`, `lib/session-queries.ts`, `chrome.tsrx`, `verify-banner.tsrx`, `__root.tsrx` |
| Tests | Unit + integration for session cache; stack browser e2e for `/status` + `auth.me` dedupe | `session-queries.unit.test.ts`, `session-cache.integration.test.ts`, `auth-ui.stack.browser.test.tsx` |
| Octane authoring | Prefer `function Page() @{` + `@if` / `@else` / `@for`; avoid React `return (` mixed with Rivet in the same component (Vite import-protection break) | `.tsrx` routes/components; see https://octanejs.dev/llms.txt |

**Explicit non-goals still deferred:** domain Zustand global store; replacing all form `useState` with Query mutations; AGENTS.md / Octane skill (repo agent docs — optional follow-up outside this addendum).
