---
status: testing
phase: 05-cloud-verify-reset
source:
  - 05-VERIFICATION.md
started: 2026-09-10T23:00:12Z
updated: 2026-09-11T16:19:00Z
---

## Current Test

number: 4
name: Disabled New repository CTA + no-invite signup
expected: |
  Unverified → disabled CTA + verify hint; verified → disabled + later-phase hint;
  signup has no invite fields
awaiting: user response

## Tests

### 1. Verify banner chrome
expected: Bottom-right verify callout (not a full-width header strip) with Resend + Enter code; Base UI toast on resend; hidden after verify; no layout shift; readable light/dark
result: pass

### 2. /verify OTP chrome
expected: 8 empty OTP slots; Working… on submit/resend; AuthShell stays; AuthErrorBanner with next step; wrap without mid-word truncate
result: pass

### 3. Anti-enumeration reset panel + redeem
expected: Unknown email → identical success panel (never not-found); redeem OTP/token UX; SSO mode shows IdP-only message
result: pass
note: |
  Automated proof: API anti-enumeration + reset-password.integration.test.ts + copy unit test.

### 4. Disabled New repository CTA + no-invite signup
expected: Unverified → disabled CTA + verify hint; verified → disabled + later-phase hint; signup has no invite fields
result: pending
note: |
  Automated proof added 2026-09-11:
  - Web: signed-in-home.integration.test.ts (unverified verify-hint + verified later-phase)
  - Web: signup.integration.test.ts (no invite UI)
  - API: signup_open_without_invite_fields_auth05

## Summary

total: 4
passed: 3
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
