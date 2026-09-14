---
schema_version: 1
open_count: 46
waived_count: 0
fixed_count: 1
total_count: 47
last_updated: 2026-09-14T15:54:38.280Z
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
| 17 | 08 | stub | crates/octanest-api/src/pat/mod.rs |  | create_fine_grained stub → pat.not_implemented (08-05) | fixed |  | 2026-09-13T18:37:52.435Z | 2026-09-13T18:42:52.268Z |
| 18 | 08 | skipped-test | crates/octanest-api/tests/git_smart_http.rs |  | 4 expansion git_smart tests #[ignore] until 08-06 | open |  | 2026-09-13T18:37:52.519Z |  |
| 19 | 08 | deviation | crates/octanest-api/src/pat/mod.rs |  | Plan prose ona_fg_ locked to octanest_fg_ (FINE_GRAINED_PAT_PREFIX / D-08) | open |  | 2026-09-13T18:42:34.877Z |  |
| 20 | 08 | unrun-verify | scripts/smoke-git-https.sh |  | Live make smoke-git-https not run — Docker engine unavailable on executor host | open |  | 2026-09-13T19:06:10.084Z |  |
| 21 | 08 | skipped-test | apps/web/src/routes/settings/tokens.integration.test.ts |  | D-15 one-time reveal it.skip until 08-10 | open |  | 2026-09-13T19:22:14.417Z |  |
| 22 | 08 | deviation | apps/web/src/components/settings/pat-revoke-dialog.tsrx |  | Revoke dialog landed with T1 list commit; T2 greened assertions | open |  | 2026-09-13T19:22:14.499Z |  |
| 23 | 09 | stub | crates/octanest-api/tests/ssh_key_rpc.rs |  | Wave 0 assert!(false) sshKey RPC stubs until 09-03 | open |  | 2026-09-13T23:34:41.937Z |  |
| 24 | 09 | stub | crates/octanest-api/tests/git_ssh.rs |  | Wave 0 assert!(false) git_ssh stubs until 09-04/09-05 | open |  | 2026-09-13T23:34:42.044Z |  |
| 25 | 09 | stub | crates/octanest-db/tests/dialect_ssh_keys.rs |  | Wave 0 dialect_ssh_keys until 0009_ssh_keys migration (09-02) | open |  | 2026-09-13T23:34:42.151Z |  |
| 26 | 09 | stub | scripts/smoke-git-ssh.sh |  | Wave 0 smoke-git-ssh exit 1 until 09-05 Compose TCP green | open |  | 2026-09-13T23:34:42.250Z |  |
| 27 | 09 | stub | apps/web/src/routes/settings/ssh-keys.integration.test.ts |  | Wave 0 RED ssh-keys integration stubs until 09-07 | open |  | 2026-09-13T23:38:48.019Z |  |
| 28 | 09 | stub | apps/web/src/components/repo/clone-box.ssh.integration.test.ts |  | Wave 0 RED CloneBox SSH integration stubs until 09-08 | open |  | 2026-09-13T23:38:48.151Z |  |
| 29 | 09 | unrun-verify | apps/web/src/routes/settings/ssh-keys.integration.test.ts |  | Wave 0 vitest intentionally RED (exit 1) until production routes — verify ran, stubs fail by design | open |  | 2026-09-13T23:38:48.269Z |  |
| 30 | 09 | stub | scripts/smoke-git-ssh.sh |  | RESOLVED: smoke-git-ssh greened in 09-05 | open |  | 2026-09-14T00:11:24.344Z |  |
| 31 | 10 | stub | apps/web/src/routes/orgs.new.integration.test.ts |  | Fails until /orgs/new lands in 10-13 | open |  | 2026-09-13T23:41:25.124Z |  |
| 32 | 10 | stub | apps/web/src/routes/new.owner-picker.integration.test.ts |  | Fails until owner Select lands in 10-11 | open |  | 2026-09-13T23:41:25.222Z |  |
| 33 | 10 | stub | apps/web/src/routes/$owner.$repo.settings.collaborators.integration.test.ts |  | Fails until collaborators-panel + can_admin in 10-11 | open |  | 2026-09-13T23:41:25.316Z |  |
| 34 | 10 | stub | apps/web/src/routes/$owner.settings.members.integration.test.ts |  | Fails until members/invites UI in 10-10 | open |  | 2026-09-13T23:41:25.415Z |  |
| 35 | 10 | stub | crates/octanest-db/migrations/postgres/0010_orgs_acl.sql |  | organization_invites/repository_collaborators tables exist without CRUD helpers/RPCs (deferred 10-06/10-07) | open |  | 2026-09-13T23:52:49.281Z |  |
| 36 | 10 | deviation | crates/octanest-api/src/auth/local.rs |  | Signup still does not dual-check organizations.slug for shared namespace (D-ORG-01); org.create does — defer to signup/rename plans | open |  | 2026-09-14T00:06:48.676Z |  |
| 37 | 10 | skipped-test | crates/octanest-api/tests/repo_collaborators_acl.rs |  | Collaborator CRUD ACL tests ignored until plan 07 | open |  | 2026-09-14T00:36:39.442Z |  |
| 38 | 10 | skipped-test | crates/octanest-api/tests/repo_private_404.rs | 410 | repo_private_404_collaborator_granted_read ignored until plan 07 | open |  | 2026-09-14T00:36:39.533Z |  |
| 39 | 10 | skipped-test | crates/octanest-api/tests/git_smart_http.rs | 610 | git_smart_collaborator_classic_pat_push Wave-0 stub fails under test(collab) filter; PAT collaborator push deferred to later plan | open |  | 2026-09-14T01:15:41.319Z |  |
| 40 | 11 | stub | crates/octanest-api/tests/issue_lifecycle.rs |  | Wave 0 assert!(false) issue lifecycle stubs until 11-02+ | open |  | 2026-09-14T14:17:20.331Z |  |
| 41 | 11 | stub | crates/octanest-db/tests/dialect_issues.rs |  | Wave 0 dialect_issues RED until 0011_issues lands | open |  | 2026-09-14T14:17:20.413Z |  |
| 42 | 11 | stub | crates/octanest-db/tests/factory_reset_issues.rs |  | Wave 0 factory_reset_issues RED until cascade wipe lands | open |  | 2026-09-14T14:17:20.496Z |  |
| 43 | 11 | deviation | crates/octanest-api/tests/git_ssh.rs |  | Rule 3: fixed insert_repository owner_type arity to unblock nextest list | open |  | 2026-09-14T14:17:20.577Z |  |
| 44 | 11 | stub | apps/web/src/routes/$owner.$repo.issues.integration.test.ts |  | Wave 0 it.fails Issues UI stubs pending 11-03..11-09 greens | open |  | 2026-09-14T14:24:49.338Z |  |
| 45 | 11 | stub | apps/web/src/lib/markdown.issues.test.ts |  | Wave 0 it.fails #N autolink stubs pending 11-10 greens | open |  | 2026-09-14T14:24:49.422Z |  |
| 46 | 11 | stub | apps/web/src/routes/$owner.$repo.issues.$n.tsrx |  | Comments/Labels/Assignees/Linked PRs empty shells until later plans | open |  | 2026-09-14T14:53:54.977Z |  |
| 47 | 11 | deviation | apps/web/src/routes/$owner.$repo.issues.$n.tsrx |  | Rule 2: wired detail route owner/repo into renderGfm despite plan 'without editing detail route files' | open |  | 2026-09-14T15:54:38.280Z |  |

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
  },
  {
    "id": 17,
    "kind": "stub",
    "phase": "08",
    "file": "crates/octanest-api/src/pat/mod.rs",
    "line": null,
    "description": "create_fine_grained stub → pat.not_implemented (08-05)",
    "status": "fixed",
    "reason": "",
    "recorded_at": "2026-09-13T18:37:52.435Z",
    "resolved_at": "2026-09-13T18:42:52.268Z"
  },
  {
    "id": 18,
    "kind": "skipped-test",
    "phase": "08",
    "file": "crates/octanest-api/tests/git_smart_http.rs",
    "line": null,
    "description": "4 expansion git_smart tests #[ignore] until 08-06",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T18:37:52.519Z",
    "resolved_at": null
  },
  {
    "id": 19,
    "kind": "deviation",
    "phase": "08",
    "file": "crates/octanest-api/src/pat/mod.rs",
    "line": null,
    "description": "Plan prose ona_fg_ locked to octanest_fg_ (FINE_GRAINED_PAT_PREFIX / D-08)",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T18:42:34.877Z",
    "resolved_at": null
  },
  {
    "id": 20,
    "kind": "unrun-verify",
    "phase": "08",
    "file": "scripts/smoke-git-https.sh",
    "line": null,
    "description": "Live make smoke-git-https not run — Docker engine unavailable on executor host",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T19:06:10.084Z",
    "resolved_at": null
  },
  {
    "id": 21,
    "kind": "skipped-test",
    "phase": "08",
    "file": "apps/web/src/routes/settings/tokens.integration.test.ts",
    "line": null,
    "description": "D-15 one-time reveal it.skip until 08-10",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T19:22:14.417Z",
    "resolved_at": null
  },
  {
    "id": 22,
    "kind": "deviation",
    "phase": "08",
    "file": "apps/web/src/components/settings/pat-revoke-dialog.tsrx",
    "line": null,
    "description": "Revoke dialog landed with T1 list commit; T2 greened assertions",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T19:22:14.499Z",
    "resolved_at": null
  },
  {
    "id": 23,
    "kind": "stub",
    "phase": "09",
    "file": "crates/octanest-api/tests/ssh_key_rpc.rs",
    "line": null,
    "description": "Wave 0 assert!(false) sshKey RPC stubs until 09-03",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:34:41.937Z",
    "resolved_at": null
  },
  {
    "id": 24,
    "kind": "stub",
    "phase": "09",
    "file": "crates/octanest-api/tests/git_ssh.rs",
    "line": null,
    "description": "Wave 0 assert!(false) git_ssh stubs until 09-04/09-05",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:34:42.044Z",
    "resolved_at": null
  },
  {
    "id": 25,
    "kind": "stub",
    "phase": "09",
    "file": "crates/octanest-db/tests/dialect_ssh_keys.rs",
    "line": null,
    "description": "Wave 0 dialect_ssh_keys until 0009_ssh_keys migration (09-02)",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:34:42.151Z",
    "resolved_at": null
  },
  {
    "id": 26,
    "kind": "stub",
    "phase": "09",
    "file": "scripts/smoke-git-ssh.sh",
    "line": null,
    "description": "Wave 0 smoke-git-ssh exit 1 until 09-05 Compose TCP green",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:34:42.250Z",
    "resolved_at": null
  },
  {
    "id": 27,
    "kind": "stub",
    "phase": "09",
    "file": "apps/web/src/routes/settings/ssh-keys.integration.test.ts",
    "line": null,
    "description": "Wave 0 RED ssh-keys integration stubs until 09-07",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:38:48.019Z",
    "resolved_at": null
  },
  {
    "id": 28,
    "kind": "stub",
    "phase": "09",
    "file": "apps/web/src/components/repo/clone-box.ssh.integration.test.ts",
    "line": null,
    "description": "Wave 0 RED CloneBox SSH integration stubs until 09-08",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:38:48.151Z",
    "resolved_at": null
  },
  {
    "id": 29,
    "kind": "unrun-verify",
    "phase": "09",
    "file": "apps/web/src/routes/settings/ssh-keys.integration.test.ts",
    "line": null,
    "description": "Wave 0 vitest intentionally RED (exit 1) until production routes — verify ran, stubs fail by design",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:38:48.269Z",
    "resolved_at": null
  },
  {
    "id": 30,
    "kind": "stub",
    "phase": "09",
    "file": "scripts/smoke-git-ssh.sh",
    "line": null,
    "description": "RESOLVED: smoke-git-ssh greened in 09-05",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T00:11:24.344Z",
    "resolved_at": null
  },
  {
    "id": 31,
    "kind": "stub",
    "phase": "10",
    "file": "apps/web/src/routes/orgs.new.integration.test.ts",
    "line": null,
    "description": "Fails until /orgs/new lands in 10-13",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:41:25.124Z",
    "resolved_at": null
  },
  {
    "id": 32,
    "kind": "stub",
    "phase": "10",
    "file": "apps/web/src/routes/new.owner-picker.integration.test.ts",
    "line": null,
    "description": "Fails until owner Select lands in 10-11",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:41:25.222Z",
    "resolved_at": null
  },
  {
    "id": 33,
    "kind": "stub",
    "phase": "10",
    "file": "apps/web/src/routes/$owner.$repo.settings.collaborators.integration.test.ts",
    "line": null,
    "description": "Fails until collaborators-panel + can_admin in 10-11",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:41:25.316Z",
    "resolved_at": null
  },
  {
    "id": 34,
    "kind": "stub",
    "phase": "10",
    "file": "apps/web/src/routes/$owner.settings.members.integration.test.ts",
    "line": null,
    "description": "Fails until members/invites UI in 10-10",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:41:25.415Z",
    "resolved_at": null
  },
  {
    "id": 35,
    "kind": "stub",
    "phase": "10",
    "file": "crates/octanest-db/migrations/postgres/0010_orgs_acl.sql",
    "line": null,
    "description": "organization_invites/repository_collaborators tables exist without CRUD helpers/RPCs (deferred 10-06/10-07)",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-13T23:52:49.281Z",
    "resolved_at": null
  },
  {
    "id": 36,
    "kind": "deviation",
    "phase": "10",
    "file": "crates/octanest-api/src/auth/local.rs",
    "line": null,
    "description": "Signup still does not dual-check organizations.slug for shared namespace (D-ORG-01); org.create does — defer to signup/rename plans",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T00:06:48.676Z",
    "resolved_at": null
  },
  {
    "id": 37,
    "kind": "skipped-test",
    "phase": "10",
    "file": "crates/octanest-api/tests/repo_collaborators_acl.rs",
    "line": null,
    "description": "Collaborator CRUD ACL tests ignored until plan 07",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T00:36:39.442Z",
    "resolved_at": null
  },
  {
    "id": 38,
    "kind": "skipped-test",
    "phase": "10",
    "file": "crates/octanest-api/tests/repo_private_404.rs",
    "line": 410,
    "description": "repo_private_404_collaborator_granted_read ignored until plan 07",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T00:36:39.533Z",
    "resolved_at": null
  },
  {
    "id": 39,
    "kind": "skipped-test",
    "phase": "10",
    "file": "crates/octanest-api/tests/git_smart_http.rs",
    "line": 610,
    "description": "git_smart_collaborator_classic_pat_push Wave-0 stub fails under test(collab) filter; PAT collaborator push deferred to later plan",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T01:15:41.319Z",
    "resolved_at": null
  },
  {
    "id": 40,
    "kind": "stub",
    "phase": "11",
    "file": "crates/octanest-api/tests/issue_lifecycle.rs",
    "line": null,
    "description": "Wave 0 assert!(false) issue lifecycle stubs until 11-02+",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:17:20.331Z",
    "resolved_at": null
  },
  {
    "id": 41,
    "kind": "stub",
    "phase": "11",
    "file": "crates/octanest-db/tests/dialect_issues.rs",
    "line": null,
    "description": "Wave 0 dialect_issues RED until 0011_issues lands",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:17:20.413Z",
    "resolved_at": null
  },
  {
    "id": 42,
    "kind": "stub",
    "phase": "11",
    "file": "crates/octanest-db/tests/factory_reset_issues.rs",
    "line": null,
    "description": "Wave 0 factory_reset_issues RED until cascade wipe lands",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:17:20.496Z",
    "resolved_at": null
  },
  {
    "id": 43,
    "kind": "deviation",
    "phase": "11",
    "file": "crates/octanest-api/tests/git_ssh.rs",
    "line": null,
    "description": "Rule 3: fixed insert_repository owner_type arity to unblock nextest list",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:17:20.577Z",
    "resolved_at": null
  },
  {
    "id": 44,
    "kind": "stub",
    "phase": "11",
    "file": "apps/web/src/routes/$owner.$repo.issues.integration.test.ts",
    "line": null,
    "description": "Wave 0 it.fails Issues UI stubs pending 11-03..11-09 greens",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:24:49.338Z",
    "resolved_at": null
  },
  {
    "id": 45,
    "kind": "stub",
    "phase": "11",
    "file": "apps/web/src/lib/markdown.issues.test.ts",
    "line": null,
    "description": "Wave 0 it.fails #N autolink stubs pending 11-10 greens",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:24:49.422Z",
    "resolved_at": null
  },
  {
    "id": 46,
    "kind": "stub",
    "phase": "11",
    "file": "apps/web/src/routes/$owner.$repo.issues.$n.tsrx",
    "line": null,
    "description": "Comments/Labels/Assignees/Linked PRs empty shells until later plans",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T14:53:54.977Z",
    "resolved_at": null
  },
  {
    "id": 47,
    "kind": "deviation",
    "phase": "11",
    "file": "apps/web/src/routes/$owner.$repo.issues.$n.tsrx",
    "line": null,
    "description": "Rule 2: wired detail route owner/repo into renderGfm despite plan 'without editing detail route files'",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-14T15:54:38.280Z",
    "resolved_at": null
  }
]
````
