# Phase 10 deferred items

## Open

- **Signup/rename shared slug dual-check:** `auth.signup` and username rename still only check `users.username`, not `organizations.slug`. `org.create` already dual-checks (D-ORG-01 / T-10-04). Close when touching signup/profile rename plans so users cannot claim an existing org slug.
