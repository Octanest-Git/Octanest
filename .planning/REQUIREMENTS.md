# Requirements: Octanest

**Defined:** 2026-09-08
**Core Value:** One forge you can trust in the cloud or on your own machines — without splitting into separate “hosted brand” vs “self-host software” products.

## v1 Requirements

Requirements for the GitHub-shaped first release. Each maps to roadmap phases later.

### Platform & delivery

- [ ] **PLAT-01**: Operator can run the full Octanest stack with Docker Compose locally
- [ ] **PLAT-02**: Operator can deploy the same images/stack to a container host (e.g. Railway) as Octanest Cloud
- [ ] **PLAT-03**: Project CI builds and validates Docker Compose (bring-up health) on every PR
- [ ] **PLAT-04**: Web UI is implemented with OctaneJS
- [ ] **PLAT-05**: Backend forge/API services are implemented in Rust
- [ ] **PLAT-06**: API is exposed as a typed RPC layer from Rust (rspc/specta-style); OctaneJS consumes a generated TypeScript client; local development regenerates client/types on change (watch-friendly)
- [ ] **PLAT-07**: Operator can configure the instance to use SQLite, PostgreSQL, or MySQL for application data
- [ ] **PLAT-08**: Migrations and core app flows work on all three supported database dialects
- [ ] **PLAT-09**: Project CI exercises at least PostgreSQL and SQLite; MySQL is either in CI or covered by an explicit compatibility test job

### Authentication & accounts

- [ ] **AUTH-01**: User can sign up with email and password
- [ ] **AUTH-02**: User can log in with email and password and stay logged in across browser refresh
- [ ] **AUTH-03**: User can log out from the web UI
- [ ] **AUTH-04**: On Octanest Cloud, user must verify email before privileged actions (at minimum: create repository)
- [ ] **AUTH-05**: On Octanest Cloud, signup is open (no invite required)
- [ ] **AUTH-06**: On self-host, if `OCTANEST_ADMIN_EMAIL` and `OCTANEST_ADMIN_PASSWORD` are both set, first boot creates that admin account
- [ ] **AUTH-07**: On self-host, if those env vars are absent, empty instance shows a one-time setup wizard to create the admin
- [ ] **AUTH-08**: User can view and edit their own profile (display name, avatar, bio)

### Git hosting

- [ ] **GIT-01**: User can create a repository (public or private)
- [ ] **GIT-02**: User can clone, fetch, and push over HTTPS with credentials
- [ ] **GIT-03**: User can clone, fetch, and push over SSH with a registered public key
- [ ] **GIT-04**: User can add, list, and revoke SSH public keys on their account
- [ ] **GIT-05**: User can browse files, commits, branches, and tags in the web UI
- [ ] **GIT-06**: User can create, rename, and delete branches from the web UI (where permitted)
- [ ] **GIT-07**: User can download a source archive for a ref
- [ ] **GIT-08**: Repository objects are stored on the local filesystem (volume-backed in Compose/cloud)
- [ ] **GIT-09**: Git operations are implemented primarily via gitoxide (pure Rust)
- [ ] **GIT-10**: Architecture docs and code boundaries allow swapping to a `git` CLI backend if gitoxide cannot meet smart HTTP/SSH compatibility

### Organizations & permissions

- [ ] **ORG-01**: User can create an organization and invite/add members
- [ ] **ORG-02**: Org owner can assign member roles that control repo access
- [ ] **ORG-03**: Repo owner can set visibility (public/private) and collaborator permissions
- [ ] **ORG-04**: Unauthorized users cannot read private repos or push without permission

### Pull requests & review

- [ ] **PR-01**: User can open a pull request from a branch (same repo or fork)
- [ ] **PR-02**: User can view PR diff, commits, and conversation
- [ ] **PR-03**: User can comment on a PR (general and line comments)
- [ ] **PR-04**: User can request changes / approve a PR
- [ ] **PR-05**: User with permission can merge a PR (merge commit, squash, or rebase — at least one strategy in v1; document which)
- [ ] **PR-06**: User can close or reopen a PR

### Issues

- [ ] **ISS-01**: User can create, edit, close, and reopen issues
- [ ] **ISS-02**: User can comment on issues
- [ ] **ISS-03**: User can assign labels and assignees to issues
- [ ] **ISS-04**: User can link issues and PRs by reference

### Actions (CI for hosted repos)

- [ ] **ACT-01**: Repo can define workflows in a GitHub Actions–compatible YAML layout
- [ ] **ACT-02**: Push and pull_request events can trigger workflow runs
- [ ] **ACT-03**: User can view workflow run status and logs in the UI
- [ ] **ACT-04**: Operator can register and run at least one Actions-compatible runner against an instance

### Packages & registry

- [ ] **PKG-01**: User can publish and pull container images from an instance registry scoped to a repo or org
- [ ] **PKG-02**: User can publish and pull at least one non-container package type (e.g. npm or generic) OR document container-only for v1 if scoped down at roadmap time
- [ ] **PKG-03**: Registry packages respect the same auth/visibility rules as their owning repo/org

### Social & explore

- [ ] **SOC-01**: User can star and unstar repositories
- [ ] **SOC-02**: User can view another user’s public profile and public repositories
- [ ] **SOC-03**: Anonymous or signed-in user can browse an explore/discover page of public repositories
- [ ] **SOC-04**: User can fork a public repository they can read

### Brand & experience

- [ ] **BRAND-01**: Product UI uses the Octanest mark (`brand/octanest-mark.png`) and blue/orange-on-dark brand direction
- [ ] **BRAND-02**: Product is named and presented as Octanest (not a “GitHub clone” or bare Octane)

## v2 Requirements

Deferred; not in the current roadmap until promoted.

### Authentication

- **AUTH-V2-01**: OAuth login (GitHub and/or Google)
- **AUTH-V2-02**: Two-factor authentication
- **AUTH-V2-03**: Password reset via email link (promote earlier if needed before public cloud launch)

### Collaboration

- **COLLAB-V2-01**: Project boards / kanban
- **COLLAB-V2-02**: Wiki per repository
- **COLLAB-V2-03**: CODEOWNERS-enforced reviews
- **COLLAB-V2-04**: Merge queues

### Platform

- **PLAT-V2-01**: Federation / ActivityPub
- **PLAT-V2-02**: Fine-grained PATs and OAuth apps for third-party integrations
- **PLAT-V2-03**: Official mobile apps

## Out of Scope

| Feature | Reason |
|---------|--------|
| Custom VCS replacing git | Must stay git-compatible |
| Forking Gitea/Forgejo as product identity | Octanest is its own brand and direction |
| Separate cloud-only vs self-host feature forks | One product, one release train |
| Vercel as forge app runtime | Stateful git needs containers; Docker is the unit |
| OAuth in v1 | Explicitly deferred after email/password |
| Invite-only cloud gate in v1 | Cloud is open signup + email verify |

## Traceability

Filled during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| *(pending roadmap)* | — | Pending |

**Coverage:**
- v1 requirements: 51 total
- Mapped to phases: 0
- Unmapped: 51 — will map in roadmap

---
*Requirements defined: 2026-09-08*
*Last updated: 2026-09-08 after multi-dialect DB support*
