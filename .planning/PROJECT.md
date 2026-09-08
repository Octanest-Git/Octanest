# Octanest

## What This Is

Octanest is a GitHub competitor: a social coding platform for hosting git repositories, collaborating on code (pull requests, issues, reviews), and shipping software together. The same product runs two ways — as a public cloud (Octanest Cloud) and as self-hosted software anyone can run on their own infrastructure.

## Core Value

One forge you can trust in the cloud or on your own machines — without splitting into separate “hosted brand” vs “self-host software” products.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet — ship to validate)

### Active

<!-- Current scope. Building toward these. -->

- [ ] Web UI: OctaneJS via `@octanejs/tanstack-start`, ShadCN + Base UI, Tailwind CSS v4 (CSS-based config)
- [ ] Theme: light + dark modes; default follows system; user can force light or dark
- [ ] Backend services in Rust
- [ ] Shared types: Rust is source of truth; TypeScript client/types via RPC codegen (rspc/specta-style) with watch-friendly regen in development
- [ ] App data store supports SQLite, PostgreSQL, and MySQL (operator-selected via config)
- [ ] Git object layer prefers pure Rust (gitoxide) on filesystem storage; real `git` CLI documented as fallback if needed
- [ ] Email/password signup, login, logout, and persistent sessions
- [ ] Email delivery: log/dev sink when unconfigured; SMTP and Resend when configured
- [ ] Email verification required before privileged cloud actions (e.g. create repos)
- [ ] Open signup on Octanest Cloud (no invite gate in v1)
- [ ] Self-host admin bootstrap: env credentials if set, otherwise one-time setup wizard
- [ ] Git hosting over SSH and HTTPS (clone, push, pull, browse)
- [ ] Users, organizations, and repository permissions
- [ ] Pull requests with review and merge
- [ ] Issues and basic project collaboration
- [ ] Actions-compatible CI with official runner image; protocol open to Blacksmith-class providers (no managed cloud minutes in v1)
- [ ] Git LFS, releases, rename/transfer, in-repo search
- [ ] Branch protection, outbound webhooks, in-app notifications
- [ ] Packages / container registry (OCI + npm + generic/raw)
- [ ] Public explore / social surface (stars, profiles, discovery)
- [ ] Octanest Cloud hosted as the same Docker stack (e.g. Railway)
- [ ] Local / self-host via Docker Compose
- [ ] Project CI validates Docker Compose (build + bring-up) on every PR

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Replacing git with a custom VCS — stay git-compatible so existing workflows work
- Forking Gitea/Forgejo as the product identity — Octanest is its own brand and codebase direction
- Separate cloud-only vs self-host-only feature forks — dual-mode means one product
- Vercel as the forge app runtime — git/SSH/stateful services need containers; Docker is the unit of deploy
- OAuth in v1 — email/password first; OAuth deferred

## Context

- **Brand:** Octanest = octane (performance / octa wink) + nest (where repos live). Chosen over Octabase (collides with AFFiNE’s OctoBase) and Octahub (too GitHub-formula + OctoHub collisions).
- **Logo (current):** [`brand/octanest-mark.png`](../brand/octanest-mark.png) — four-arrow X mark, blue (cool/left) + orange (warm/right) on black. Use as primary mark (UI, favicon, README) until a vector set exists.
- **Brand colors (from mark):** cool blues/cyans vs warm oranges; high-contrast on dark surfaces.
- **Model:** GitLab-style dual-mode (one product, cloud + self-host), not Codeberg/Forgejo split (hosted instance vs different software brand).
- **Stack direction:** **Rust** backend services; **OctaneJS** web via [`@octanejs/tanstack-start`](https://github.com/octanejs/octane/tree/main/packages/tanstack-start). UI kit: **ShadCN + Base UI + Tailwind CSS v4** (CSS-based config). Theme: system default with light/dark override. Rust owns API/domain types; TypeScript consumes generated RPC types. Do not name this product bare “Octane.”
- **Source control (working):** [`git@github.com:Octanest-Git/Octanest.git`](https://github.com/Octanest-Git/Octanest) — org `Octanest-Git`, repo `Octanest`.
- **Name availability (2026-09-08, informational):**
  - npm `octanest`: free
  - crates.io `octanest`: free
  - Docker Hub `library/octanest`: not found (free to claim)
  - GitHub org [`octanest`](https://github.com/octanest): taken (unrelated); we use `Octanest-Git` instead
  - `octanest.com`: DNS resolves (consumer/LLP brand elsewhere)
  - `octanest.dev` / `.io` / `.app`: no DNS at check time (candidates)

## Constraints

- **Brand:** Keep the octa/octane connection; never ship as “GitHub clone” branding
- **Distribution:** Cloud and self-host must share one codebase and one Docker-based release train
- **Compatibility:** Real git clients and remotes must work (`git@…:user/repo.git`)
- **Runtime:** Docker Compose is the supported way to run Octanest (local, self-host, and cloud)
- **Cloud host:** Container platform such as Railway (same images as local Compose)
- **Self-host bootstrap:** If `OCTANEST_ADMIN_EMAIL` and `OCTANEST_ADMIN_PASSWORD` are set, create that admin on first boot. Otherwise show a one-time setup wizard to create the admin, then normal signup rules apply for the instance.
- **UI:** OctaneJS on TanStack Start (`@octanejs/tanstack-start`); ShadCN + Base UI components; Tailwind CSS v4 with CSS-first configuration
- **Theme:** Light and dark; default = system preference; user can lock light or dark
- **Backend:** Rust for forge/API/git-facing services
- **Type safety:** Rust → TypeScript via RPC + codegen (rspc / specta-style); procedures and types stay in sync with watch-friendly regen in development
- **App database:** SQLite, PostgreSQL, and MySQL all supported via one storage abstraction; instance chooses dialect through config/env
- **Git engine:** Prefer **gitoxide** (pure Rust) with repos on filesystem volumes; keep **`git` CLI + filesystem** as an explicit fallback path if protocol/compat gaps block progress
- **Email:** Unconfigured → log/dev sink; configured → **SMTP** and **Resend** adapters
- **Project CI:** Docker Compose build + validation on every PR (the path that ships)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Product name: **Octanest** | Octane DNA + nest metaphor; softer collisions than Octabase/Octahub | ✓ Good |
| Dual-mode (cloud + self-host) | Compete with GitHub while letting people run their own instance | ✓ Good |
| One brand for both modes | Avoid Codeberg/Forgejo-style brand split | ✓ Good |
| GitHub remote: **Octanest-Git/Octanest** | Working source-control home while product brand is Octanest | ✓ Good |
| v1 bar: **GitHub-shaped slice** | Thin forge + Actions CI + packages/registry + explore/social | — Pending |
| Runtime: **Docker Compose everywhere** | One container stack for local, self-host, and cloud | ✓ Good |
| Cloud host: **Railway (or equiv.)** | Host the same Compose/images; not a second app architecture | — Pending |
| Dropped: **Vercel as forge runtime** | Stateful git needs containers; Docker is enough | ✓ Good |
| Project CI: **Compose validation** | CI proves the path operators actually run | — Pending |
| Auth v1: **email + password** | Ship sessions first; OAuth later | ✓ Good |
| Cloud signup: **open + email verify** | Public competitor feel; verify before privileged actions | ✓ Good |
| Self-host admin: **wizard, or env if set** | Compose-friendly override; safe default for empty installs | ✓ Good |
| UI: **OctaneJS + TanStack Start** | `@octanejs/tanstack-start` app shell | ✓ Good |
| UI kit: **ShadCN + Base UI + Tailwind v4** | CSS-based Tailwind config; component system | ✓ Good |
| Theme: **system default, light/dark override** | Respect OS; allow explicit preference | ✓ Good |
| Backend: **Rust** | Performance/correctness for git-heavy forge services | ✓ Good |
| Types: **RPC + codegen (rspc/specta-style)** | End-to-end procedure + type safety; watch regen in dev | ✓ Good |
| App DB: **SQLite + Postgres + MySQL** | Operator choice; one abstraction, three dialects | ✓ Good |
| Git engine: **gitoxide preferred** | Pure Rust; `git` CLI fallback documented if compat fails | ✓ Good |
| Email: **log sink / SMTP / Resend** | Safe default locally; real relays when configured | ✓ Good |
| Git HTTPS: **PATs only** | No account password over git; create/revoke in UI | ✓ Good |
| PR merges: **merge / squash / rebase** | All three strategies; per-repo enable/disable | ✓ Good |
| Packages: **OCI + npm + generic/raw** | Full registry surface for common publish needs | ✓ Good |
| Actions: **official runner image + 3rd-party protocol** | Operator-hosted compute; Blacksmith-class providers can integrate; no managed minutes in v1 | ✓ Good |
| v1 extras: **LFS, webhooks, notifs, search, releases, transfer, branch protection** | Full GitHub-shaped collaboration surface | ✓ Good |
| Logo: **brand/octanest-mark.png** | Current brand mark (blue/orange X) | ✓ Good |

---
*Last updated: 2026-09-08 after locking TanStack Start + ShadCN/BaseUI/Tailwind v4 theme*
