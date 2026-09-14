# Phase 14: Git LFS - Context

**Gathered:** 2026-09-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Operators get volume-backed Git LFS storage; users can push/fetch LFS objects over HTTPS for repos that enable it. Delivers GIT-12, GIT-13.

**Requirements:** GIT-12, GIT-13

**Success criteria (from ROADMAP):**
1. Operator can configure filesystem (volume-backed) LFS storage for the instance
2. User can push and fetch Git LFS objects for a repository

**Out of scope (later / deferred):**
- LFS-over-SSH (Phase 9 deferred; not in Phase 14)
- Managed/cloud object backends (S3, etc.) — filesystem volume only for v1
- Auto-committing `.gitattributes` on enable
- Closing the gap where missing server OIDs fail smudge (standard LFS error path only)

**UI hint:** yes — repo Settings LFS toggle/status/usage; Admin quotas/usage; blob pointer badge + Download; LFS browser surfaces.

</domain>

<decisions>
## Implementation Decisions

### A — Storage layout
- **D-LFS-01:** Separate instance volume via **`OCTANEST_LFS_DIR`** (Compose bind like uploads), not under each bare repo — **Reversibility:** costly — ops path + backup story
- **D-LFS-02:** **Instance-wide content-addressed OID store** (dedup across repos) — **Reversibility:** costly — shared GC/refcount model
- **D-LFS-03:** On-disk layout **OID-sharded** (`ab/cd/<oid>`) plus **DB pointer/refcount rows** — **Reversibility:** costly — migration if changed
- **D-LFS-04:** **Factory reset wipes `LFS_DIR`** along with repositories — **Reversibility:** reversible (policy)

### B — Transport
- **D-LFS-05:** **HTTPS only** in Phase 14 (batch + content transfer); **LFS-over-SSH deferred** — **Reversibility:** reversible (add later)
- **D-LFS-06:** Routes under **`/{owner}/{repo}.git/info/lfs/…`** (reuse `.git` Traefik → API routing) — **Reversibility:** costly — client URL expectations
- **D-LFS-07:** Ship **basic transfer + multipart/resumable uploads** — **Reversibility:** reversible (multipart is additive)
- **D-LFS-08:** **Git clone without LFS smudge remains valid** (pointer files in tree) — GitHub/Gitea parity — **Reversibility:** reversible

### C — Auth & ACL
- **D-LFS-09:** Auth matches Smart HTTP: **PAT Basic**; **Read** for download, **Write** for upload; **no session cookies** for LFS — **Reversibility:** reversible
- **D-LFS-10:** **Per-repo enable/disable**; only **Admin** may toggle — **Reversibility:** reversible
- **D-LFS-11:** **Reuse existing contents/repo PAT scopes** — no dedicated `lfs` fine-grained permission — **Reversibility:** reversible

### D — Operator & limits
- **D-LFS-12:** **Configurable max object size** and **per-repo + per-user storage quotas** in Phase 14 — **Reversibility:** costly — quota schema + enforcement
- **D-LFS-13:** **Instance env defaults** with **Admin UI overrides** — **Reversibility:** reversible
- **D-LFS-14:** **Reject** uploads that exceed max size or quota (clear error; no soft-warn-only) — **Reversibility:** reversible
- **D-LFS-15:** **Refcount + periodic GC** of unreferenced OIDs — **Reversibility:** costly — GC job + safety

### E — Client UX
- **D-LFS-16:** Ship **docs + repo Settings** (toggle/status) **+ pointer badges on blob/tree + in-app LFS browser / quota dashboards** — **Reversibility:** costly — UI surface area
- **D-LFS-17:** **Document `.gitattributes` patterns only** — no server auto-commit of attributes — **Reversibility:** reversible
- **D-LFS-18:** Blob view **detects LFS pointer** and offers **Download via LFS** (AuthZ = Read) — **Reversibility:** reversible
- **D-LFS-19:** Usage dashboards at **repo settings** (this repo) and **Admin** (instance); both include **usage breakdown** — **Reversibility:** reversible

### Claude's Discretion
- Exact default max object size and default quota numbers
- Exact multipart chunk size / resume protocol details (within Git LFS-compatible basic+multipart)
- Exact GC schedule and locking
- Exact breakdown dimensions (by repo, user, OID count, bytes) as long as both dashboards show a useful breakdown
- Exact Settings copy for enable LFS + link to docs

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning / requirements
- `.planning/ROADMAP.md` — Phase 14 goal, GIT-12, GIT-13
- `.planning/REQUIREMENTS.md` — GIT-12, GIT-13 wording
- `.planning/phases/08-git-https-pats/08-CONTEXT.md` — Smart HTTP PAT auth / scopes
- `.planning/phases/09-git-ssh/09-CONTEXT.md` — LFS-over-SSH deferred note
- `.planning/phases/10-orgs-permissions/10-CONTEXT.md` — Read/Write/Admin ACL

### Code mirrors
- `crates/octanest-api/src/routes/git_smart_http.rs` — PAT Basic + pack ACL patterns to mirror
- `crates/octanest-api/src/git/http_backend.rs` — CGI / `OCTANEST_REPOS_DIR` pattern
- `crates/octanest-api/src/repo/acl.rs` — capability checks for LFS enable + transfer
- `docker-compose.yml` — `.git` Traefik route + volume bind pattern for new `LFS_DIR`
- `docs/CONFIGURATION.md` / `docs/API.md` — git auth contracts to extend for LFS

### Specs (external)
- Git LFS Batch API / basic transfer — https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md (planner/researcher should pin a revision)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Smart HTTP PAT auth + rate limit + ACL
- `OCTANEST_REPOS_DIR` / `var/uploads` volume pattern for a new LFS volume
- Repo settings UI patterns; Admin settings patterns
- Blob viewer — extend for pointer detection + Download

### Established Patterns
- Cookies ignored for git HTTPS; PAT Basic only
- Soft ACL / capability gates via `repo/acl.rs`
- Factory reset already clears repos — extend to LFS_DIR

### Integration Points
- New Axum routes under `/{owner}/{repo}.git/info/lfs/…`
- Repo settings → LFS enable + usage breakdown
- Admin → instance quotas/usage breakdown
- Compose + CONFIGURATION docs for `OCTANEST_LFS_DIR` and limits

</code_context>

<specifics>
## Specific Ideas

- Prefer GitHub/Gitea parity for smudge-skip / pointer clones
- User explicitly wants rich UI in Phase 14: badges, browser, and quota dashboards with breakdown — not docs-only

</specifics>

<deferred>
## Deferred Ideas

- LFS-over-SSH
- S3 / external object stores
- Auto-commit starter `.gitattributes`
- Rejecting git clone when LFS objects missing (non-parity)

</deferred>

---

*Phase: 14-git-lfs*
*Context gathered: 2026-09-14*
