---
status: testing
phase: 07-git-repos-browse
source:
  - 07-VERIFICATION.md
started: "2026-09-12T20:04:02Z"
updated: "2026-09-12T20:04:02Z"
---

## Current Test

number: 1
name: Syntax highlighting fidelity (07-00 backstop)
expected: |
  Tokens highlight via in-repo grammars (not plain TS/JS alias look) for .tsrx and .ripple blobs
awaiting: user response

## Tests

### 1. Syntax highlighting fidelity (07-00 backstop)
expected: Open a seeded blob for `.tsrx` and `.ripple` in the Code UI — tokens highlight via in-repo grammars (not plain TS/JS alias look)
result: pending

### 2. /new description wrap (07-03 backstop)
expected: Paste a long unbroken description on `/new` — text wraps; no horizontal page overflow
result: pending

### 3. Long path ellipsis (07-15 backstop)
expected: Browse a deep/long file path in tree/blob chrome — ellipsis or wrap per UI-SPEC; layout remains usable
result: pending

### 4. Clone/download box (07-08)
expected: On Code tab, clone box shows HTTPS + SSH placeholder + archive menu items
result: pending

### 5. Highlight + README sanitize (07-15)
expected: Open a seeded repo blob for `.ts` / `.tsrx` — highlight works; script tags stripped from README render
result: pending

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps
