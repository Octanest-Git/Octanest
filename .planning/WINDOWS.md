---
schema_version: 1
open_count: 9
waived_count: 0
fixed_count: 0
total_count: 9
last_updated: 2026-09-12T18:23:10.268Z
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
  }
]
````
