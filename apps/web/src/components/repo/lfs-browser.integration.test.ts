import { describe, expect, it } from "vitest";

/**
 * D-LFS-16: in-app LFS browser surface for enabled repos.
 */
describe("LFS browser (D-LFS-16)", () => {
  it(
    "exposes in-app OID/list surface for LFS-enabled repositories",
    async () => {
      const mod = await import("./lfs-browser");
      expect(
        mod.LfsBrowser ?? mod.default,
        "LfsBrowser component must exist for in-app OID browser (D-LFS-16)",
      ).toBeTruthy();
      const src = await import("./lfs-browser.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(src).toMatch(/listObjects/);
      expect(src).toMatch(/download/i);
      expect(src).toMatch(/enabled/);
    },
    30_000,
  );
});
