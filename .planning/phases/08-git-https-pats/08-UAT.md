---
status: testing
phase: 08-git-https-pats
source: [08-VERIFICATION.md]
started: 2026-09-13T20:08:56Z
updated: 2026-09-13T20:08:56Z
---

## Current Test

number: 1
name: Signed-in verified user opens /settings/tokens, creates a classic PAT, copies the one-time reveal, then revokes it from the list
expected: |
  List empty hero → Generate → classic form → reveal once → list shows prefix/note → revoke confirm removes token; password never accepted as git secret
awaiting: user response

## Tests

### 1. Tokens UI — create classic / reveal / revoke in a real browser
expected: List empty hero → Generate → classic form → reveal once → list shows prefix/note → revoke confirm removes token; password never accepted as git secret
result: [pending]

### 2. FG + real git HTTPS — Compose smoke
expected: Token mints with octanest_fg_; Basic auth with PAT works; account password fails with PAT hint; private anon gets 401+WWW-Authenticate
result: [pending]

### 3. How-to panel + long URL wrap — visual backstop
expected: How-to lists username aliases git/token/oauth2, password=PAT, Create CTA → /settings/tokens; long URL does not clip awkwardly
result: [pending]

### 4. Judgment prohibitions — plaintext-at-rest / no RPC Bearer ack
expected: DB only stores token_hash; /api/rpc still session-only; docs say PATs are not RPC Bearer
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
