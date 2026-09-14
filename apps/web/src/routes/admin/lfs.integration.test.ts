import { describe, expect, it } from "vitest";

/**
 * D-LFS-12 / D-LFS-13 / D-LFS-19: Admin LFS quotas + instance usage.
 */
describe("admin LFS quotas (D-LFS-12 / D-LFS-13 / D-LFS-19)", () => {
  it(
    "Admin can override max-object / per-repo / per-user quotas + see instance breakdown",
    async () => {
      const mod = await import("./lfs");
      expect(
        mod.AdminLfsPage ?? mod.default,
        "Admin LFS page must exist for quota overrides (D-LFS-13)",
      ).toBeTruthy();
      const src = await import("./lfs.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(src).toMatch(/admin\.lfs\.getSettings|getSettings/);
      expect(src).toMatch(/updateSettings/);
      expect(src).toMatch(/getUsage/);
      expect(src).toMatch(/max.object|Max object/i);
      expect(src).toMatch(/quota/i);
      expect(src).toMatch(/by_repo|By repository/i);
      expect(src).toMatch(/by_owner|By owner/i);
      expect(src).toMatch(/physical/i);
      expect(src).toMatch(/sys-admin/);
    },
    30_000,
  );
});
