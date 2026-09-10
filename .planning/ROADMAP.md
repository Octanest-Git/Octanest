# Roadmap: Octanest

## Overview

Octanest ships as a GitHub-shaped forge — one Rust + Octane TanStack Start codebase, ShadCN/Base UI/Tailwind v4, one Docker Compose release train, dual-mode cloud and self-host. This fine-grained roadmap builds from monorepo scaffold and multi-DB storage through branded auth (with system/light/dark theme), git hosting (HTTPS/SSH), orgs and collaboration (issues, PRs, protection), then LFS/releases/search, notifications/webhooks, Actions, packages, social explore, and finally Compose CI plus Railway deploy. Each phase delivers one verifiable capability; parallelizable slices are noted in dependencies.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Monorepo Scaffold** - Rust + `@octanejs/tanstack-start` + Compose + ShadCN/Base UI/Tailwind v4 + typed RPC codegen
- [x] **Phase 2: Multi-DB Storage** - SQLite, PostgreSQL, and MySQL via one storage abstraction
- [x] **Phase 3: Brand Shell & Theme** - Octanest mark, chrome, light/dark with system default (completed 2026-09-09)
- [x] **Phase 4: Auth Sessions & Email** - Signup, login, logout, sessions, profile, email providers (completed 2026-09-10)
- [ ] **Phase 5: Cloud Verify & Reset** - Open cloud signup, email verify gate, password reset
- [ ] **Phase 6: Self-Host Admin Bootstrap** - Env admin or one-time setup wizard
- [ ] **Phase 7: Git Repos & Browse** - gitoxide filesystem repos, create, browse, branches, archives
- [ ] **Phase 8: Git HTTPS & PATs** - Smart HTTP clone/push with personal access tokens
- [ ] **Phase 9: Git SSH** - SSH keys and clone/fetch/push over SSH
- [ ] **Phase 10: Orgs & Permissions** - Organizations, roles, visibility, access enforcement
- [ ] **Phase 11: Issues** - Create, comment, labels, assignees, issue↔PR links
- [ ] **Phase 12: Pull Requests** - Open, review, comment, merge strategies, close/reopen
- [ ] **Phase 13: Branch Protection** - Protection rules enforced on push and merge
- [ ] **Phase 14: Git LFS** - LFS push/fetch with volume-backed storage
- [ ] **Phase 15: Releases & Transfer** - Releases/assets, rename, and transfer repos
- [ ] **Phase 16: In-Repo Search** - Search code, commits, issues, and PRs in a repo
- [ ] **Phase 17: Notifications** - In-app notifications for issue and PR activity
- [ ] **Phase 18: Webhooks** - Outbound webhooks, delivery, and attempt history
- [ ] **Phase 19: Actions & Runners** - Actions-compatible CI, official runner, open protocol
- [ ] **Phase 20: Packages Registry** - OCI, npm, and generic/raw packages with auth
- [ ] **Phase 21: Social & Explore** - Stars, profiles, explore, and forks
- [ ] **Phase 22: Compose CI & Cloud Deploy** - PR Compose matrix and Railway-class deploy path

## Phase Details

### Phase 1: Monorepo Scaffold

**Goal**: Developers and operators can bring up a Rust backend + Octane TanStack Start frontend via Docker Compose with ShadCN/Base UI/Tailwind v4 and typed RPC codegen regenerating in local development
**Depends on**: Nothing (first phase)
**Requirements**: PLAT-01, PLAT-04, PLAT-05, PLAT-06, PLAT-10, PLAT-11
**Success Criteria** (what must be TRUE):

  1. Operator can start the Octanest stack with Docker Compose and reach a healthy web UI and API
  2. Web UI is an OctaneJS app on TanStack Start (`@octanejs/tanstack-start`)
  3. UI uses ShadCN + Base UI components and Tailwind CSS v4 with CSS-based configuration
  4. Backend forge/API process is a Rust service reachable from the UI
  5. Changing a Rust RPC procedure regenerates the TypeScript client/types in watch-friendly local development

**Plans**:

- [x] `01-01-PLAN.md` — Monorepo skeleton (Bun/Turborepo + Cargo + Makefile)
- [x] `01-02-PLAN.md` — Axum RPC + specta codegen + HTTP/WS tests
- [x] `01-03-PLAN.md` — Octane Start web + UI-SPEC landing/status
- [x] `01-04-PLAN.md` — Docker Compose + Traefik + smoke
- [x] `01-05-PLAN.md` — CI gates + VALIDATION nyquist

**UI hint**: yes

### Phase 2: Multi-DB Storage

**Goal**: Operators can choose SQLite, PostgreSQL, or MySQL for app data with working migrations and core flows on all three
**Depends on**: Phase 1
**Requirements**: PLAT-07, PLAT-08
**Success Criteria** (what must be TRUE):

  1. Operator can configure the instance dialect to SQLite, PostgreSQL, or MySQL via config/env
  2. Migrations apply cleanly on each supported dialect
  3. A core app write/read flow succeeds against each dialect in a local Compose setup

**Plans**:

- [x] `02-01-PLAN.md` — octanest-db connection layer (dialect resolution, redaction, `DbPool`/`Database` connect)
- [x] `02-02-PLAN.md` — per-dialect migrations, `instances` probe, migrate CLI, integration tests
- [x] `02-03-PLAN.md` — API startup fail-fast + auto-migrate + `system.db_probe` RPC + generated client
- [x] `02-04-PLAN.md` — SQLite Compose overlay, Make targets, dialect-asserting smoke, dialect switch
- [x] `02-05-PLAN.md` — CI `db-matrix` (3 dialects) + operator docs + validation sign-off

### Phase 3: Brand Shell & Theme

**Goal**: The product UI reads as Octanest — mark, colors, naming — with light/dark themes defaulting to system and user override
**Depends on**: Phase 1
**Requirements**: BRAND-01, BRAND-02, BRAND-03, BRAND-04, BRAND-05
**Success Criteria** (what must be TRUE):

  1. Primary chrome (favicon, header/nav mark) uses `brand/octanest-mark.png` and blue/orange brand direction
  2. Product copy and titles present the name Octanest (not “GitHub clone” or bare Octane)
  3. UI renders correctly in light and dark modes
  4. Theme defaults to the OS preference (system) until the user chooses light or dark
  5. User can set theme to light or dark and the preference persists across refresh

**Plans**: 6 plans

- [x] `03-01-PLAN.md` — semantic token layer + CVA Button/Input/Select + shared squircle OctanestMark
- [x] `03-02-PLAN.md` — branded header/footer chrome, theme Select, pre-paint FOUC boot script
- [x] `03-03-PLAN.md` — four-band editorial landing refresh with three reduced-motion-aware motions
- [x] `03-04-PLAN.md` — branded `/status` hero states + `Status · Octanest` title
- [x] `03-05-PLAN.md` — favicon/app-icon set, Vite PWA manifest, assets-only service worker
- [x] `03-06-PLAN.md` — phase-wide gates + human verification of light/dark, system default, persistence

**UI hint**: yes

### Phase 4: Auth Sessions & Email

**Goal**: Users can create accounts, stay signed in, manage a basic profile, and operators can send mail via log sink, SMTP, or Resend
**Depends on**: Phase 2, Phase 3
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-08, AUTH-09, AUTH-10, AUTH-11
**Success Criteria** (what must be TRUE):

  1. User can sign up with email and password
  2. User can log in and remain logged in across browser refresh, and can log out from the web UI
  3. User can view and edit their own profile (display name, avatar, bio)
  4. With no email provider configured, outbound mail appears in a log/dev sink; with SMTP or Resend configured, mail is sent through that provider

**Plans**: 8/8 plans executed

- [x] `04-01-PLAN.md` — Auth schema, DTOs, and DB CRUD (users/sessions/identities/settings)
- [x] `04-02-PLAN.md` — EmailSender log-sink + SMTP + Resend adapters
- [x] `04-03-PLAN.md` — Argon2id passwords + opaque session cookies
- [x] `04-04-PLAN.md` — Local auth RPC, welcome email, admin seed, session tests
- [x] `04-05-PLAN.md` — WorkOS AuthKit + generic OIDC callbacks
- [x] `04-06-PLAN.md` — Profile/avatar API + admin.auth settings + rpc-gen
- [x] `04-07-PLAN.md` — Login/signup/dashboard UI + chrome account menu
- [x] `04-08-PLAN.md` — Profile + admin auth UI + human UAT checkpoint

**UI hint**: yes

### Phase 5: Cloud Verify & Reset

**Goal**: Octanest Cloud feels open to the public while requiring email verification before privileged actions and supporting password reset
**Depends on**: Phase 4
**Requirements**: AUTH-04, AUTH-05, AUTH-12
**Success Criteria** (what must be TRUE):

  1. On Octanest Cloud, signup requires no invite
  2. On Octanest Cloud, unverified users cannot perform privileged actions (at minimum: create repository) until email is verified
  3. When an email provider is configured, user can reset password via an email link

**Plans**: 7/7 plans executed

- [x] 05-01-PLAN.md
- [x] 05-02-PLAN.md
- [x] 05-03-PLAN.md
- [x] 05-04-PLAN.md
- [x] 05-05-PLAN.md
- [x] 05-06-PLAN.md
- [x] 05-07-PLAN.md

- [ ] `05-01-PLAN.md` — Token schema/CRUD + UserPublic.email_verified field
- [ ] `05-02-PLAN.md` — Tracer: OTP verify → require_verified / privileged_ping
- [ ] `05-03-PLAN.md` — Verify issue/resend/rate limits, signup auto-send, admin seed, AUTH-05
- [ ] `05-04-PLAN.md` — Password reset request/redeem RPCs (AUTH-12)
- [ ] `05-05-PLAN.md` — IdP-trust verified marking + clear_email_verification helper
- [ ] `05-06-PLAN.md` — rpc-gen, input-otp gate, `/verify`, VerifyBanner
- [ ] `05-07-PLAN.md` — `/reset-password`, forgot link, disabled New repository CTA

**UI hint**: yes

### Phase 6: Self-Host Admin Bootstrap

**Goal**: Empty self-host installs get a first admin via env credentials or a one-time setup wizard
**Depends on**: Phase 4
**Requirements**: AUTH-06, AUTH-07
**Success Criteria** (what must be TRUE):

  1. On self-host, when `OCTANEST_ADMIN_EMAIL` and `OCTANEST_ADMIN_PASSWORD` are both set, first boot creates that admin account
  2. On self-host, when those env vars are absent, an empty instance shows a one-time wizard to create the admin, then continues with normal signup rules

**Plans**: TBD
**UI hint**: yes

### Phase 7: Git Repos & Browse

**Goal**: Users can create filesystem-backed repos and browse history in the UI, powered primarily by gitoxide with a documented CLI fallback boundary
**Depends on**: Phase 5, Phase 6
**Requirements**: GIT-01, GIT-05, GIT-06, GIT-07, GIT-08, GIT-09, GIT-10
**Success Criteria** (what must be TRUE):

  1. Authenticated (and verified, on cloud) user can create a public or private repository
  2. User can browse files, commits, branches, and tags in the web UI and download a source archive for a ref
  3. User can create, rename, and delete branches from the web UI where permitted
  4. Repository objects live on the local filesystem (volume-backed), and git operations use gitoxide with architecture docs allowing a `git` CLI backend swap

**Plans**: TBD
**UI hint**: yes

### Phase 8: Git HTTPS & PATs

**Goal**: Users can authenticate git over HTTPS with personal access tokens (never account passwords) and manage those tokens in the UI
**Depends on**: Phase 7
**Requirements**: GIT-02, GIT-11
**Success Criteria** (what must be TRUE):

  1. User can create, list, and revoke personal access tokens for HTTPS git (and RPC/API where applicable)
  2. User can clone, fetch, and push over HTTPS using a PAT; account password is rejected for git auth

**Plans**: TBD
**UI hint**: yes

### Phase 9: Git SSH

**Goal**: Users can register SSH keys and clone/fetch/push over SSH like a normal forge remote
**Depends on**: Phase 7
**Requirements**: GIT-03, GIT-04
**Success Criteria** (what must be TRUE):

  1. User can add, list, and revoke SSH public keys on their account
  2. User can clone, fetch, and push over SSH with a registered public key

**Plans**: TBD
**UI hint**: yes

### Phase 10: Orgs & Permissions

**Goal**: Users can collaborate via organizations and repository permissions with private data truly private
**Depends on**: Phase 7
**Requirements**: ORG-01, ORG-02, ORG-03, ORG-04
**Success Criteria** (what must be TRUE):

  1. User can create an organization and invite/add members
  2. Org owner can assign member roles that control repo access; repo owner can set visibility and collaborator permissions
  3. Unauthorized users cannot read private repos or push without permission

**Plans**: TBD
**UI hint**: yes

### Phase 11: Issues

**Goal**: Users can track work with issues, comments, labels, assignees, and links to PRs
**Depends on**: Phase 10
**Requirements**: ISS-01, ISS-02, ISS-03, ISS-04
**Success Criteria** (what must be TRUE):

  1. User can create, edit, close, and reopen issues
  2. User can comment on issues and assign labels and assignees
  3. User can link issues and PRs by reference

**Plans**: TBD
**UI hint**: yes

### Phase 12: Pull Requests

**Goal**: Users can open, review, and merge pull requests with configurable merge strategies
**Depends on**: Phase 8, Phase 9, Phase 10, Phase 11
**Requirements**: PR-01, PR-02, PR-03, PR-04, PR-05, PR-06, PR-07
**Success Criteria** (what must be TRUE):

  1. User can open a pull request from a branch (same repo or fork) and view diff, commits, and conversation
  2. User can leave general and line comments and request changes or approve
  3. User with permission can merge choosing merge commit, squash, or rebase; can close or reopen a PR
  4. Repo settings can enable/disable each merge strategy independently

**Plans**: TBD
**UI hint**: yes

### Phase 13: Branch Protection

**Goal**: Repo admins can require reviews and/or status checks before merge, enforced on direct pushes and PR merges
**Depends on**: Phase 12
**Requirements**: ORG-05, ORG-06, PR-08
**Success Criteria** (what must be TRUE):

  1. Repo admin can configure branch protection rules (e.g. require reviews and/or status checks)
  2. Protected rules block non-compliant direct pushes
  3. PR merge is blocked when applicable branch protection rules are not satisfied

**Plans**: TBD
**UI hint**: yes

### Phase 14: Git LFS

**Goal**: Users can push and fetch LFS objects with operator-configured volume-backed LFS storage
**Depends on**: Phase 8, Phase 9
**Requirements**: GIT-12, GIT-13
**Success Criteria** (what must be TRUE):

  1. Operator can configure LFS storage on the filesystem (volume-backed) for the instance
  2. User can push and fetch Git LFS objects for a repository

**Plans**: TBD

### Phase 15: Releases & Transfer

**Goal**: Users can ship tagged releases with assets and rename or transfer repositories they administer
**Depends on**: Phase 10
**Requirements**: GIT-14, GIT-15, GIT-16, GIT-17
**Success Criteria** (what must be TRUE):

  1. User can create a release for a tag with notes and downloadable assets, and download those assets from the web UI
  2. User with permission can rename a repository
  3. User with permission can transfer a repository to another user or organization

**Plans**: TBD
**UI hint**: yes

### Phase 16: In-Repo Search

**Goal**: Users can find code, commits, issues, and PRs inside repositories they can read
**Depends on**: Phase 11, Phase 12
**Requirements**: GIT-18
**Success Criteria** (what must be TRUE):

  1. User can search code and commits within a repository they can read
  2. User can search issues and PRs within a repository they can read

**Plans**: TBD
**UI hint**: yes

### Phase 17: Notifications

**Goal**: Signed-in users stay aware of issue and PR activity via in-app notifications
**Depends on**: Phase 11, Phase 12
**Requirements**: NOTF-01, NOTF-02
**Success Criteria** (what must be TRUE):

  1. Signed-in user receives in-app notifications for relevant issue and PR activity
  2. User can list notifications and mark them as read

**Plans**: TBD
**UI hint**: yes

### Phase 18: Webhooks

**Goal**: Repo admins can subscribe outbound webhooks and inspect delivery attempts
**Depends on**: Phase 11, Phase 12
**Requirements**: HOOK-01, HOOK-02, HOOK-03
**Success Criteria** (what must be TRUE):

  1. Repo admin can create, edit, and delete outbound webhooks for repo events
  2. Instance delivers webhook payloads for subscribed events (at least push, PR, and issue events)
  3. Repo admin can view recent webhook delivery attempts and response status

**Plans**: TBD
**UI hint**: yes

### Phase 19: Actions & Runners

**Goal**: Repos can run GitHub Actions–compatible workflows on operator-registered runners (official image + open third-party protocol), with no managed cloud minutes in v1
**Depends on**: Phase 12, Phase 13
**Requirements**: ACT-01, ACT-02, ACT-03, ACT-04, ACT-05, ACT-06, ACT-07
**Success Criteria** (what must be TRUE):

  1. Repo can define workflows in a GitHub Actions–compatible YAML layout; push and pull_request events trigger runs
  2. User can view workflow run status and logs in the UI
  3. Operator can register and run the official Octanest runner image; docs cover Compose sidecar or standalone bring-up
  4. Forge exposes an Actions-compatible registration/job-dispatch protocol (custom `runs-on` labels); jobs only run on registered runners — no managed Octanest Cloud minutes in v1

**Plans**: TBD
**UI hint**: yes

### Phase 20: Packages Registry

**Goal**: Users can publish and pull OCI, npm, and generic/raw packages scoped to repo/org with the same auth/visibility rules
**Depends on**: Phase 10
**Requirements**: PKG-01, PKG-02, PKG-03, PKG-04, PKG-05
**Success Criteria** (what must be TRUE):

  1. User can publish and pull OCI container images from an instance registry scoped to a repo or org
  2. User can publish and pull npm packages and generic/raw packages from an instance registry scoped to a repo or org
  3. Registry packages respect the same auth/visibility rules as their owning repo/org
  4. User can list and delete package versions they are permitted to manage

**Plans**: TBD
**UI hint**: yes

### Phase 21: Social & Explore

**Goal**: Users can discover public work via explore, profiles, stars, and forks
**Depends on**: Phase 10, Phase 12
**Requirements**: SOC-01, SOC-02, SOC-03, SOC-04
**Success Criteria** (what must be TRUE):

  1. User can star and unstar repositories
  2. User can view another user’s public profile and public repositories
  3. Anonymous or signed-in user can browse an explore/discover page of public repositories
  4. User can fork a public repository they can read

**Plans**: TBD
**UI hint**: yes

### Phase 22: Compose CI & Cloud Deploy

**Goal**: Every PR proves Compose health across supported databases, and the same images deploy as Octanest Cloud on a container host
**Depends on**: Phase 2, Phase 19, Phase 20, Phase 21
**Requirements**: PLAT-02, PLAT-03, PLAT-09
**Success Criteria** (what must be TRUE):

  1. Project CI builds and validates Docker Compose (bring-up health) on every PR
  2. Project CI exercises at least PostgreSQL and SQLite; MySQL is in CI or covered by an explicit compatibility test job
  3. Operator can deploy the same images/stack to a container host (e.g. Railway) as Octanest Cloud

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → … → 22

**Parallelism notes:** After Phase 7, Phases 8 and 9 can proceed in parallel. After Phase 10, Phases 11 and 14–15/20 can overlap where staffing allows. After Phase 12, Phases 16–18 are largely independent. Phase 22 waits on multi-DB (Phase 2) and product completeness of Actions/packages/social for a meaningful cloud path.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Monorepo Scaffold | 0/TBD | Not started | - |
| 2. Multi-DB Storage | 0/4 | Planned | - |
| 3. Brand Shell & Theme | 6/6 | Complete    | 2026-09-09 |
| 4. Auth Sessions & Email | 8/8 | Complete    | 2026-09-10 |
| 5. Cloud Verify & Reset | 7/7 | In Progress|  |
| 6. Self-Host Admin Bootstrap | 0/TBD | Not started | - |
| 7. Git Repos & Browse | 0/TBD | Not started | - |
| 8. Git HTTPS & PATs | 0/TBD | Not started | - |
| 9. Git SSH | 0/TBD | Not started | - |
| 10. Orgs & Permissions | 0/TBD | Not started | - |
| 11. Issues | 0/TBD | Not started | - |
| 12. Pull Requests | 0/TBD | Not started | - |
| 13. Branch Protection | 0/TBD | Not started | - |
| 14. Git LFS | 0/TBD | Not started | - |
| 15. Releases & Transfer | 0/TBD | Not started | - |
| 16. In-Repo Search | 0/TBD | Not started | - |
| 17. Notifications | 0/TBD | Not started | - |
| 18. Webhooks | 0/TBD | Not started | - |
| 19. Actions & Runners | 0/TBD | Not started | - |
| 20. Packages Registry | 0/TBD | Not started | - |
| 21. Social & Explore | 0/TBD | Not started | - |
| 22. Compose CI & Cloud Deploy | 0/TBD | Not started | - |

---
*Roadmap created: 2026-09-09*
*Granularity: fine — 22 phases, 85/85 v1 requirements mapped*
