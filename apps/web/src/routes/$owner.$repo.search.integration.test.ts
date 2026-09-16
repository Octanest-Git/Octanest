import { describe, expect, it } from "vitest";

/**
 * Phase 16 / GIT-18 Wave 0 — in-repo search route UI (D-SRCH-02, D-SRCH-15).
 * Greened across 16-01 (code tracer) and 16-03 (full tabs + chrome entry).
 *
 * Type tabs: Code | Commits | Issues | Pull requests
 * Query input + empty/truncated states.
 */

describe("repo search route (GIT-18 / D-SRCH-02 / D-SRCH-15)", () => {
  it.todo("renders query input for /{owner}/{repo}/search");

  it.todo("exposes type tabs: Code, Commits, Issues, Pull requests");

  it.todo("shows code hits from repo.search type=code");

  it.todo("switches type query param and refetches on tab change");

  it.todo("renders empty and truncated states");
});
