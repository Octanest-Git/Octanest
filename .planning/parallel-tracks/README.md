# Remaining-phases integration track

**Integrate branch:** `cursor/gsd-remaining-integrate-c82f`  
**Umbrella PR:** [#16](https://github.com/Octanest-Git/Octanest/pull/16) → `main` after manual UAT  
**Phase PRs:** each targets this integrate branch (not `main`)

## Merge gates (required)

Each phase PR is **standalone** and must be reviewed on its own.

**Do not merge a phase PR into `integrate` until all of:**

1. **CI green** on that PR’s head branch  
2. **Human review** approved (or explicitly waived by a maintainer)  
3. PR is mergeable onto current `integrate` (rebase if needed)

Agents may continue implementing on phase branches in parallel; landing into `integrate` is gated as above. Do **not** fast-forward/`git merge` locally to skip GitHub CI/review.

## Merge order into integrate (after gates)

1. Phase 12 (Pull Requests) — unlocks the rest  
2. Parallel: 13, 16, 17, 18, 21  
3. Phase 19 (after 13)  
4. Phase 22 (after 19 + 21)

Do not merge the umbrella PR to `main` until product UAT on the integrate stack.
