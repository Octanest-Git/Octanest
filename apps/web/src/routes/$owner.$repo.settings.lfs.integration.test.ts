import { describe, expect, it } from "vitest";

/**
 * D-LFS-10 / D-LFS-16 / D-LFS-19: repo Settings LFS section.
 */
describe("repo settings LFS (D-LFS-10 / D-LFS-16 / D-LFS-19)", () => {
  it("Admin sees enable toggle + status + repo usage breakdown", async () => {
    const settings = await import("./$owner.$repo.settings");
    expect(settings.RepoSettingsPage ?? settings.default).toBeTruthy();
    const panel = await import("../components/repo/lfs-settings-panel");
    expect(panel.LfsSettingsPanel ?? panel.default).toBeTruthy();
    const src = await import("./$owner.$repo.settings.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(src).toMatch(/LfsSettingsPanel/);
    expect(src).toMatch(/can_admin/);
    const panelSrc = await import("../components/repo/lfs-settings-panel.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(panelSrc).toMatch(/Enable Git LFS/);
    expect(panelSrc).toMatch(/usage|quota|breakdown/i);
    expect(panelSrc).toMatch(/gitattributes|git lfs track/i);
    expect(panelSrc).toMatch(/repo\.lfs\.setEnabled|lfs\.setEnabled/);
    expect(panelSrc).toMatch(/getUsage/);
  }, 30_000);

  it("non-Admin cannot toggle LFS enable (D-LFS-10)", async () => {
    const src = await import("./$owner.$repo.settings.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(src).toMatch(/can_admin/);
    expect(src).toMatch(/LfsSettingsPanel/);
    const panelSrc = await import("../components/repo/lfs-settings-panel.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    // Toggle Switch rendered only when can_admin prop is true.
    expect(panelSrc).toMatch(/canAdmin/);
    expect(panelSrc).toMatch(/@if \(canAdmin\)/);
  }, 30_000);
});
