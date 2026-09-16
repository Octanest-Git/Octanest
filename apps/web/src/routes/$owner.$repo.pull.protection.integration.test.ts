import { describe, expect, it } from "vitest";

/**
 * PR-08 / D-22 / D-24: PR detail merge-blocker display (Wave 0 stub).
 * Greened by 13-08 once Phase 12 PR detail is extended.
 */

describe("PR merge protection blockers (PR-08 / D-24)", () => {
  it.todo(
    "lists unmet requirements: reviews, checks, conversations, up-to-date, locked",
  );

  it("documents expected blocker reason keys for 13-08", () => {
    const reasons = [
      "reviews",
      "checks",
      "conversations",
      "up_to_date",
      "locked",
    ] as const;
    expect(reasons).toContain("reviews");
    expect(reasons).toContain("checks");
    expect(reasons).toContain("conversations");
    expect(reasons).toContain("up_to_date");
    expect(reasons).toContain("locked");
  });
});
