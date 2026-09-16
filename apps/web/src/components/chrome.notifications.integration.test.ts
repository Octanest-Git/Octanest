/**
 * Phase 17 Wave 0 — SiteHeader notifications bell (D-08 / 17-UI-SPEC / NOTF-02).
 * Greened by 17-04 when bell + unread badge land.
 */
import { describe, expect, it } from "vitest";

describe("SiteHeader notifications bell Wave 0 (D-08 / NOTF-02)", () => {
  it("signed-in chrome exposes a notifications control with unread affordance", async () => {
    const chromeSrc = await import("./chrome.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(
      chromeSrc,
      "Wave 0: SiteHeader must expose a notifications bell linking to /notifications (D-08)",
    ).toMatch(/notifications/i);
    expect(
      chromeSrc,
      "Wave 0: bell must surface unread count affordance (D-08 / 17-UI-SPEC)",
    ).toMatch(/unread/i);
  });
});
