import { describe, expect, it } from "vitest";

/**
 * Wave 0 / D-LFS-10 / D-LFS-16 / D-LFS-19: repo Settings LFS section.
 * Greened when Settings LFS UI ships (14-09).
 */
describe("repo settings LFS (D-LFS-10 / D-LFS-16 / D-LFS-19)", () => {
  it.skip(
    "Admin sees enable toggle + status + repo usage breakdown",
    async () => {
      const settings = await import("./$owner.$repo.settings");
      expect(settings.RepoSettingsPage ?? settings.default).toBeTruthy();
      const src = await import("./$owner.$repo.settings.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(src).toMatch(/LFS|lfs/);
      expect(src).toMatch(/enable|toggle/i);
      expect(src).toMatch(/usage|quota|breakdown/i);
    },
    30_000,
  );

  it.skip(
    "non-Admin cannot toggle LFS enable (D-LFS-10)",
    async () => {
      const src = await import("./$owner.$repo.settings.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      // Enable control must be gated on can_admin (not owner_id equality alone).
      expect(src).toMatch(/can_admin/);
      expect(src).toMatch(/LFS|lfs/);
    },
    30_000,
  );
});
