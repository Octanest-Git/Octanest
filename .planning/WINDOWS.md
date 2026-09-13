---
schema_version: 1
open_count: 16
waived_count: 0
fixed_count: 0
total_count: 16
last_updated: 2026-09-13T18:04:19.657Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 06 | unrun-verify | crates/octanest-api/tests/auth_bootstrap.rs |  | Plan 06-02 full test(bootstrap) filter deferred: allowlist/allow_signup Wave 0 RED owned by 06-03 | open |  | 2026-09-11T20:42:23.023Z |  |
| 2 | 07 | stub | crates/octanest-git/src/version.rs | 43 | git_archive_formats_zip_and_tar_gz still Wave 0 assert!(false) — archive plan owns | open |  | 2026-09-12T17:13:59.869Z |  |
| 3 | 07 | stub | apps/web/src/routes/new.tsrx | 206 | Stack/License/.gitignore None placeholders until 07-03 | open |  | 2026-09-12T17:20:43.199Z |  |
| 4 | 07 | stub | apps/web/src/routes/$owner.$repo.index.tsrx |  | Empty Quick setup only; tree/README deferred to 07-15 | open |  | 2026-09-12T17:20:43.278Z |  |
| 5 | 07 | deviation | apps/web/src/routes/$owner.$repo.tsrx |  | Added Outlet layout parent required for $owner.$repo.index route | open |  | 2026-09-12T17:20:43.358Z |  |
| 6 | 07 | unrun-verify | crates/octanest-git/src/version.rs | 73 | Wave 0 git_archive_formats stub fails full octanest-git --lib until archive plan; 07-05 verified with not test(git_archive) | open |  | 2026-09-12T17:59:04.501Z |  |
| 7 | 07 | stub | apps/web/src/routes/$owner.$repo.index.tsrx |  | Clone/Download toolbar stub until 07-08 | open |  | 2026-09-12T18:11:31.663Z |  |
| 8 | 07 | skipped-test | crates/octanest-git/src/version.rs |  | Pre-existing git_archive_formats_zip_and_tar_gz Wave 0 stub fails nextest | open |  | 2026-09-12T18:23:10.183Z |  |
| 9 | 07 | skipped-test | crates/octanest-api/tests/repo_branch_soft_protect.rs |  | Wave 0 soft-protect stubs owned by 07-07 | open |  | 2026-09-12T18:23:10.268Z |  |
| 10 | 08 | stub | crates/octanest-api/tests/pat_rpc.rs |  | Wave 0 RED pat_* stubs until 08-04 | open |  | 2026-09-13T17:58:59.409Z |  |
| 11 | 08 | stub | crates/octanest-api/tests/git_smart_http.rs |  | Wave 0 RED git_smart_* stubs until 08-04/08-06 | open |  | 2026-09-13T17:58:59.506Z |  |
| 12 | 08 | stub | crates/octanest-db/tests/dialect_pats.rs |  | Wave 0 dialect_pats until 08-03 0008_pats | open |  | 2026-09-13T17:58:59.595Z |  |
| 13 | 08 | deviation | crates/octanest-db/tests/dialect_pats.rs |  | Renamed dialect tests for test(dialect_pats) nextest filter | open |  | 2026-09-13T17:58:59.686Z |  |
| 14 | 08 | stub | apps/web/src/routes/settings/tokens.integration.test.ts |  | Wave 0 RED tokens UI stubs until 08-09/08-10 | open |  | 2026-09-13T18:04:19.488Z |  |
| 15 | 08 | stub | apps/web/src/components/repo/clone-box.pat.integration.test.ts |  | Wave 0 RED CloneBox PAT how-to stubs until 08-12 | open |  | 2026-09-13T18:04:19.572Z |  |
| 16 | 08 | deviation | apps/web/src/routes/settings/tokens.integration.test.ts |  | Used runtime-variable @vite-ignore import so Vitest collects while tokens route absent | open |  | 2026-09-13T18:04:19.657Z |  |

````json
[
  {
    "id": 1,
    "kind": "unrun-verify",
    "phase": "06",
    "file": "crates/octanest-api/tests/auth_bootstrap.rs",
    "line": null,
    "description": "Plan 06-02 full test(bootstrap) filter deferred: allowlist/allow_signup Wave 0 RED owned by 06-03",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-11T20:42:23.023Z",
    "resolved_at": null
  },
  {
    "id": 2,
    "kind": "stub",
    "phase": "07",
    "file": "crates/octanest-git/src/version.rs",
    "line": 43,
    "description": "git_archive_formats_zip_and_tar_gz still Wave 0 assert!(false) — archive plan owns",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:13:59.869Z",
    "resolved_at": null
  },
  {
    "id": 3,
    "kind": "stub",
    "phase": "07",
    "file": "apps/web/src/routes/new.tsrx",
    "line": 206,
    "description": "Stack/License/.gitignore None placeholders until 07-03",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:20:43.199Z",
    "resolved_at": null
  },
  {
    "id": 4,
    "kind": "stub",
    "phase": "07",
    "file": "apps/web/src/routes/$owner.$repo.index.tsrx",
    "line": null,
    "description": "Empty Quick setup only; tree/README deferred to 07-15",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:20:43.278Z",
    "resolved_at": null
  },
  {
    "id": 5,
    "kind": "deviation",
    "phase": "07",
    "file": "apps/web/src/routes/$owner.$repo.tsrx",
    "line": null,
    "description": "Added Outlet layout parent required for $owner.$repo.index route",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:20:43.358Z",
    "resolved_at": null
  },
  {
    "id": 6,
    "kind": "unrun-verify",
    "phase": "07",
    "file": "crates/octanest-git/src/version.rs",
    "line": 73,
    "description": "Wave 0 git_archive_formats stub fails full octanest-git --lib until archive plan; 07-05 verified with not test(git_archive)",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T17:59:04.501Z",
    "resolved_at": null
  },
  {
    "id": 7,
    "kind": "stub",
    "phase": "07",
    "file": "apps/web/src/routes/$owner.$repo.index.tsrx",
    "line": null,
    "description": "Clone/Download toolbar stub until 07-08",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:11:31.663Z",
    "resolved_at": null
  },
  {
    "id": 8,
    "kind": "skipped-test",
    "phase": "07",
    "file": "crates/octanest-git/src/version.rs",
    "line": null,
    "description": "Pre-existing git_archive_formats_zip_and_tar_gz Wave 0 stub fails nextest",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:23:10.183Z",
    "resolved_at": null
  },
  {
    "id": 9,
    "kind": "skipped-test",
    "phase": "07",
    "file": "crates/octanest-api/tests/repo_branch_soft_protect.rs",
    "line": null,
    "description": "Wave 0 soft-protect stubs owned by 07-07",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-12T18:23:10.268Z",
    "resolved_at": null
  },
  {
    "id": 10,
    "kind": "stub",
    "phase": "08",
    "file": "crates/octanest-api/tests/pat_rpc.rs",
    "line": null,
    "description": "Wave 0 RED pat_* stubs until 08-04",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T17:58:59.409Z",
    "resolved_at": null
  },
  {
    "id": 11,
    "kind": "stub",
    "phase": "08",
    "file": "crates/octanest-api/tests/git_smart_http.rs",
    "line": null,
    "description": "Wave 0 RED git_smart_* stubs until 08-04/08-06",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T17:58:59.506Z",
    "resolved_at": null
  },
  {
    "id": 12,
    "kind": "stub",
    "phase": "08",
    "file": "crates/octanest-db/tests/dialect_pats.rs",
    "line": null,
    "description": "Wave 0 dialect_pats until 08-03 0008_pats",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T17:58:59.595Z",
    "resolved_at": null
  },
  {
    "id": 13,
    "kind": "deviation",
    "phase": "08",
    "file": "crates/octanest-db/tests/dialect_pats.rs",
    "line": null,
    "description": "Renamed dialect tests for test(dialect_pats) nextest filter",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T17:58:59.686Z",
    "resolved_at": null
  },
  {
    "id": 14,
    "kind": "stub",
    "phase": "08",
    "file": "apps/web/src/routes/settings/tokens.integration.test.ts",
    "line": null,
    "description": "Wave 0 RED tokens UI stubs until 08-09/08-10",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T18:04:19.488Z",
    "resolved_at": null
  },
  {
    "id": 15,
    "kind": "stub",
    "phase": "08",
    "file": "apps/web/src/components/repo/clone-box.pat.integration.test.ts",
    "line": null,
    "description": "Wave 0 RED CloneBox PAT how-to stubs until 08-12",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T18:04:19.572Z",
    "resolved_at": null
  },
  {
    "id": 16,
    "kind": "deviation",
    "phase": "08",
    "file": "apps/web/src/routes/settings/tokens.integration.test.ts",
    "line": null,
    "description": "Used runtime-variable @vite-ignore import so Vitest collects while tokens route absent",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T18:04:19.657Z",
    "resolved_at": null
  }
]
````
