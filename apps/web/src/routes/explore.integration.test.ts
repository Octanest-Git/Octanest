import { describe, expect, it } from "vitest";

/**
 * SOC-03 Wave 0 — ExplorePage /explore route contract.
 * Turns green in 21-04 when explore.tsrx ships.
 */

describe("explore route (Wave 0 / SOC-03)", () => {
  it("exports ExplorePage from explore route module", async () => {
    const mod = await import("./explore");
    expect(mod.ExplorePage).toBeTypeOf("function");
  });
});
