# Phase 4: User Setup Required

**Generated:** 2026-09-09
**Phase:** 04-auth-sessions-email
**Status:** Incomplete

Complete these items for live SMTP/Resend delivery. Claude automated adapters and tests; CI uses the log sink and wiremock. These items require operator credentials.

## Environment Variables

| Status | Variable | Source | Add to |
|--------|----------|--------|--------|
| [ ] | `OCTANEST_SMTP_URL` | Operator SMTP URL e.g. `smtp://user:pass@host:587` (lettre `from_url`) | `.env` / Compose |
| [ ] | `OCTANEST_MAIL_FROM` | From address e.g. `Octanest <noreply@example.com>` (default: `Octanest <noreply@localhost>`) | `.env` / Compose |
| [ ] | `OCTANEST_RESEND_API_KEY` | Resend Dashboard → API Keys | `.env` / Compose |

## Account Setup

- [ ] **Create Resend account** (optional — only if using Resend)
  - URL: https://resend.com
  - Skip if: Using SMTP only, or local log-sink development

- [ ] **Have SMTP credentials** (optional — only if using SMTP)
  - Skip if: Using Resend only, or local log-sink development

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

# With Resend key set at runtime, signup welcome (plan 04-04) hits Resend.
# With OCTANEST_SMTP_URL set, SmtpSender is selected instead.
```

Expected results:
- Unconfigured env → LogSink only (`octanest.mail` tracing target)
- `OCTANEST_RESEND_API_KEY` set → Resend preferred over SMTP
- `OCTANEST_SMTP_URL` set (no Resend key) → lettre SMTP path

---

**Once all items complete:** Mark status as "Complete" at top of file.
