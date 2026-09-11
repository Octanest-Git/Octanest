# Deferred items — Phase 06 (recorded during 06-02)

## Out of scope for 06-02 (Wave 0 RED → later plans)

| Item | Owner | Notes |
|------|-------|-------|
| `bootstrap_strict_rpc_allowlist_while_needs_setup` | 06-03 | D-11 strict RPC while `needs_setup` |
| `bootstrap_allow_signup_false_blocks_signup` stubs | 06-03 | Wizard persist `allow_signup` + signup reject |
| `bootstrap_allow_signup_true_persists` stubs | 06-03 | Wizard `allow_signup=true` persistence |

Plan 06-02 `<verify>` used `test(bootstrap)` which includes these intentional RED stubs; Task 3 acceptance verified via `test(partial_env) | test(seeded_admin) | test(seed_*)` instead.
