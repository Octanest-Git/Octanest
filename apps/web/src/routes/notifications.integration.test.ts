/**
 * Phase 17 Wave 0 — /notifications inbox page (D-09 / D-12 / NOTF-02).
 * Greened by 17-04 when notifications.tsrx lands.
 */
import { describe, expect, it } from "vitest";

describe("/notifications Wave 0 (D-09 / D-12 / NOTF-02)", () => {
  it("page exports Unread|All filters and Mark all as read", async () => {
    let pageSrc = "";
    try {
      pageSrc = await import("./notifications.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
    } catch (err) {
      expect.fail(
        `Wave 0: /notifications route missing — implement in 17-04 (NOTF-02 / D-09). ${(err as Error).message}`,
      );
    }
    expect(pageSrc, "Wave 0: Unread filter (D-09)").toMatch(/Unread/);
    expect(pageSrc, "Wave 0: All filter (D-09)").toMatch(/\bAll\b/);
    expect(pageSrc, "Wave 0: Mark all as read (D-12 / NOTF-02)").toMatch(/Mark all as read/);
  });
});
