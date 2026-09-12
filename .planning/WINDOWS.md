---
schema_version: 1
open_count: 2
waived_count: 0
fixed_count: 0
total_count: 2
last_updated: 2026-09-12T17:13:59.869Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 06 | unrun-verify | crates/octanest-api/tests/auth_bootstrap.rs |  | Plan 06-02 full test(bootstrap) filter deferred: allowlist/allow_signup Wave 0 RED owned by 06-03 | open |  | 2026-09-11T20:42:23.023Z |  |
| 2 | 07 | stub | crates/octanest-git/src/version.rs | 43 | git_archive_formats_zip_and_tar_gz still Wave 0 assert!(false) — archive plan owns | open |  | 2026-09-12T17:13:59.869Z |  |

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
  }
]
````
