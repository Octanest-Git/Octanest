# Remaining-phases integration track

**Integrate branch:** `cursor/gsd-remaining-integrate-c82f`  
**Umbrella PR:** targets `main` after manual testing  
**Phase PRs:** each targets this integrate branch (not `main`)

## Merge order into integrate

1. Phase 12 (Pull Requests) — unlocks the rest  
2. Parallel: 13, 16, 17, 18, 21  
3. Phase 19 (after 13)  
4. Phase 22 (after 19 + 21)

Do not merge this branch to `main` until product UAT on the integrate deployment/stack.
