---
status: testing
phase: 05-cloud-verify-reset
source:
  - 05-VERIFICATION.md
started: 2026-09-10T23:00:12Z
updated: 2026-09-10T23:00:12Z
---

## Current Test

number: 1
name: Verify banner chrome
expected: |
  Sign in as unverified local user; banner under SiteHeader with message + Resend + Enter code;
  after verify, banner hidden; readable in light and dark.
awaiting: user response

## Tests

### 1. Verify banner chrome
expected: Banner under SiteHeader with message + Resend + Enter code; hidden after verify; readable light/dark
result: pending

### 2. /verify OTP chrome
expected: 8 empty OTP slots; Working… on submit/resend; AuthShell stays; AuthErrorBanner with next step; wrap without mid-word truncate
result: pending

### 3. Anti-enumeration reset panel + redeem
expected: Unknown email → identical success panel (never not-found); redeem OTP/token UX; SSO mode shows IdP-only message
result: pending

### 4. Disabled New repository CTA + no-invite signup
expected: Unverified → disabled CTA + verify hint; verified → disabled + later-phase hint; signup has no invite fields
result: pending

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
