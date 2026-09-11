---
created: 2026-09-11T01:07:22.477Z
title: Fix signed-in home flicker on load
area: ui
severity: minor
files:
  - apps/web/src/routes/index.tsrx
  - apps/web/src/lib/session-hint.ts
  - apps/web/src/components/page-skeletons.tsrx
  - apps/web/src/components/signed-in-home.tsrx
---

## Problem

Logged-out marketing home no longer throws `insertBefore` (landing paints during auth check). Signed-in `/` (“dashboard” home) still **flickers** on load: brief wrong chrome / landing-or-empty → skeleton → `SignedInHome` while `octanest_signed_in` hint + `auth.me` resolve. SSR cannot read the HttpOnly session; presence cookie heals only after mount, so first paint for returning sessions is unstable.

## Solution

Likely tighten the signed-in gate so first paint matches the final home:

- Prefer a stable signed-in shell (skeleton or content) whenever the presence cookie is present **before** paint where possible (inline boot script mirroring theme boot?).
- Avoid painting `AnonLanding` when a presence hint will flip true on the next tick.
- Consider SSR/`auth.me` in a route loader so the match commits once with the right tree.
- Re-check view-transition on `.octanest-main` interacting with gate swaps.

TBD which approach after profiling the flicker frames.
