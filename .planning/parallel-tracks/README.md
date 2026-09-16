# Remaining-phases integration track

**Integrate branch:** `cursor/gsd-remaining-integrate-c82f`  
**Umbrella PR:** [#16](https://github.com/Octanest-Git/Octanest/pull/16) → `main` after manual UAT  
**Phase PRs:** each targets this integrate branch (not `main`)

## Status (2026-09-16)

| Phase | Integrate |
|-------|-----------|
| 12, 13, 16, 17, 18, 21, 22 | Merged |
| 19 Actions & Runners | In progress on `cursor/phase-19-actions-runners-c82f` |

GitHub Actions: repo is public; hosted runners available again. Prefer green CI; local validation still required before merge.

## Merge gates (required)

Each phase PR is **standalone** and must be reviewed on its own.

**Do not merge a phase PR into `integrate` until all of:**

1. **CI green** on that PR’s head branch (or maintainer-waived while GitHub Actions is unavailable)
2. **Human review** approved (or explicitly waived by a maintainer)
3. PR is mergeable onto current `integrate` (rebase if needed)

### GitHub Actions note (2026-09-16)

Hosted runners fail instantly (`runner_id=0`, empty steps) on this private repo.
Org Actions usage for Sep 2026 is **exactly 2000 Linux minutes** (included free private quota).
Until minutes reset, spending is enabled, or self-hosted/public runners are available, **CI cannot go green**.

## Migration numbers (wave-2)

| Phase | Migration |
|-------|-----------|
| 13 Branch protection | `0017_branch_protection` |
| 17 Notifications | `0018_notifications` |
| 18 Webhooks | `0019_webhooks` |
| 21 Social | `0020_social` |
| 16 Search | (none) |

**Merge order:** 13 first (keeps 0017 contiguous), then 16 / 17 / 18 / 21 (any order among themselves after 13), then 19, then 22.

## Merge order into integrate (after gates)

1. Phase 12 — already in integrate  
2. Phase 13  
3. Parallel: 16, 17, 18, 21  
4. Phase 19 (after 13)  
5. Phase 22 (after 19 + 21)
