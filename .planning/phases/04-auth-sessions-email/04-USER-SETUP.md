# Phase 4: User Setup Required

**Generated:** 2026-09-09
**Updated:** 2026-09-09 (plan 04-04 admin seed)
**Phase:** 04-auth-sessions-email
**Status:** Incomplete

Complete these items for live SMTP/Resend delivery and optional first-admin bootstrap. Claude automated adapters, auth RPC, and tests; CI uses the log sink and wiremock. These items require operator credentials.

## Environment Variables

| Status | Variable | Source | Add to |
|--------|----------|--------|--------|
| [ ] | `OCTANEST_SMTP_URL` | Operator SMTP URL e.g. `smtp://user:pass@host:587` (lettre `from_url`) | `.env` / Compose |
| [ ] | `OCTANEST_MAIL_FROM` | From address e.g. `Octanest <noreply@example.com>` (default: `Octanest <noreply@localhost>`) | `.env` / Compose |
| [ ] | `OCTANEST_RESEND_API_KEY` | Resend Dashboard → API Keys | `.env` / Compose |
| [ ] | `OCTANEST_ADMIN_EMAIL` | Optional first-admin email when `users` is empty (before Phase 6 wizard) | `.env` / Compose |
| [ ] | `OCTANEST_ADMIN_PASSWORD` | Optional first-admin password (paired with `OCTANEST_ADMIN_EMAIL`) | `.env` / Compose |

## Account Setup

- [ ] **Create Resend account** (optional — only if using Resend)
  - URL: https://resend.com
  - Skip if: Using SMTP only, or local log-sink development

- [ ] **Have SMTP credentials** (optional — only if using SMTP)
  - Skip if: Using Resend only, or local log-sink development

- [ ] **Optional admin seed** (dev / self-host bootstrap)
  - Set both `OCTANEST_ADMIN_EMAIL` and `OCTANEST_ADMIN_PASSWORD` only when you want a single `is_admin` user created on first boot with an empty users table
  - Username becomes `admin` (or `admin1` if taken); Phase 6 owns the interactive wizard

## Dashboard Configuration

- [ ] **Verify Resend From domain** (if using Resend)
  - Location: Resend Dashboard → Domains
  - Set to: Domain matching `OCTANEST_MAIL_FROM`
  - Notes: Unverified From addresses are rejected by Resend

## Verification

After completing setup, verify with:

```bash
# Default (no provider): LogSink — no network
unset OCTANEST_SMTP_URL OCTANEST_RESEND_API_KEY
cargo test -p octanest-api --lib email::

# Auth signup/session (local mode + welcome email path)
cargo test -p octanest-api --test auth_signup --test auth_session

# With Resend key set at runtime, signup welcome hits Resend.
# With OCTANEST_SMTP_URL set, SmtpSender is selected instead.
```

Expected results:
- Unconfigured env → LogSink only (`octanest.mail` tracing target)
- `OCTANEST_RESEND_API_KEY` set → Resend preferred over SMTP
- `OCTANEST_SMTP_URL` set (no Resend key) → lettre SMTP path
- Both `OCTANEST_ADMIN_*` set + empty users → one admin user logged once at boot

---

**Once all items complete:** Mark status as "Complete" at top of file.
