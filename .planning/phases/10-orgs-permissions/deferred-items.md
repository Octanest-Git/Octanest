# Phase 10 deferred items

## Open

- **Signup/rename shared slug dual-check:** `auth.signup` and username rename still only check `users.username`, not `organizations.slug`. `org.create` already dual-checks (D-ORG-01 / T-10-04). Close when touching signup/profile rename plans so users cannot claim an existing org slug.

- [ ] 10-07: `git_smart_collaborator_classic_pat_push` Wave-0 stub still fails under `test(collab)` filter — PAT collaborator push is a later plan (out of scope for 10-07).
