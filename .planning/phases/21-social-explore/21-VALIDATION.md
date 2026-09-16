# Phase 21: Social & Explore - Validation

**Nyquist / Wave 0 checklist** — every later plan `<automated>` must name a real path from this list (or create it in 21-00).

## Wave 0 Requirements

| ID | Stub / filter | Turns green in |
|----|---------------|----------------|
| SOC-01 | `test(repo_stars)` in `crates/octanest-api/tests/repo_stars.rs` | 21-01 / 21-02 |
| SOC-02 | `test(user_public_profile)` in `crates/octanest-api/tests/user_public_profile.rs` | 21-03 |
| SOC-03 | `test(repo_explore)` in `crates/octanest-api/tests/repo_explore.rs` | 21-04 |
| SOC-04 | `test(repo_fork)` in `crates/octanest-api/tests/repo_fork.rs` | 21-05 / 21-06 |
| Dialect | `test(dialect_social)` in `crates/octanest-db/tests/dialect_social.rs` | 21-01 / 21-05 |
| Fork network helper | covered inside `repo_fork` (head_valid_for_base cases) | 21-05 |
| Web explore | `apps/web/src/routes/explore.integration.test.ts` | 21-04 |
| Web profile | extend `$owner.layout.integration.test.ts` or `user-profile.integration.test.ts` | 21-03 |
| Web chrome star/fork | `apps/web/src/components/repo/repo-chrome.social.integration.test.ts` | 21-02 / 21-06 |

## Phase gate (final plan)

- `cargo nextest -p octanest-api -E 'test(repo_stars) | test(repo_fork) | test(repo_explore) | test(user_public_profile)'`
- `cargo nextest -p octanest-db -E 'test(dialect_social)'`
- `make rpc-sync-check`
- Web: relevant vitest filters + `make web-lint` / `make web-format-check` when UI touched
- Docs: `docs/API.md` lists new procedures

## ASSUME

- Wave 0 stubs may `#[ignore]` or `assert!(false)` until implementation plans.
- Migration logical name `0016_social` — confirm against filesystem at execute time.
