import { describe, expect, it } from "vitest";

/**
 * Wave 0 / D-LFS-12 / D-LFS-13 / D-LFS-19: Admin LFS quotas + instance usage.
 * Greened when Admin LFS UI ships (14-10).
 */
describe("admin LFS quotas (D-LFS-12 / D-LFS-13 / D-LFS-19)", () => {
  it.skip(
    "Admin can override max-object / per-repo / per-user quotas + see instance breakdown",
    async () => {
      let mod: Record<string, unknown> | null = null;
      try {
        mod = await import("./lfs");
      } catch {
        try {
          mod = await import("./settings");
        } catch {
          mod = null;
        }
      }
      expect(
        mod?.default ?? mod?.AdminLfsPage ?? mod?.AdminSettingsPage,
        "Admin LFS / settings surface must exist for quota overrides (D-LFS-13)",
      ).toBeTruthy();
    },
    30_000,
  );
});
