import { describe, expect, it } from "vitest";

/**
 * Wave 0 / D-LFS-16: in-app LFS browser surface for enabled repos.
 * Greened when LFS browser component/route ships (14-11).
 */
describe("LFS browser (D-LFS-16)", () => {
  it.skip(
    "exposes in-app OID/list surface for LFS-enabled repositories",
    async () => {
      // Prefer a dedicated browser component once 14-11 adds it.
      let mod: Record<string, unknown> | null = null;
      try {
        mod = await import("./lfs-browser");
      } catch {
        mod = null;
      }
      expect(
        mod?.LfsBrowser ?? mod?.default,
        "LfsBrowser component must exist for in-app OID browser (D-LFS-16)",
      ).toBeTruthy();
    },
    30_000,
  );
});
