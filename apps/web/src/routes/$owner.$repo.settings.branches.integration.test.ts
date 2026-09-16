import { describe, expect, it } from "vitest";

/**
 * ORG-05 / D-03 / D-25: Repo Settings → Branch protection.
 */

describe("repo settings Branch protection (ORG-05 / D-25)", () => {
  it("settings gate uses can_admin for Branch protection panel", async () => {
    const panel = await import("../components/repo/branch-protection-panel");
    expect(
      panel.BranchProtectionPanel ?? panel.default,
      "BranchProtectionPanel must export for Admin-gated settings (D-25)",
    ).toBeTruthy();

    const settingsSrc = await import("./$owner.$repo.settings.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(settingsSrc).toMatch(/BranchProtectionPanel/);
    expect(settingsSrc).toMatch(/can_admin/);
  }, 30_000);

  it("Branch protection section: list + Add rule controls", async () => {
    const src = await import("../components/repo/branch-protection-panel.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(src).toMatch(/Branch protection/);
    expect(src).toMatch(/Add rule/);
    expect(src).toMatch(/branchProtection\.create/);
    expect(src).toMatch(/branchProtection\.list/);
    expect(src).toMatch(/required reviews|Require pull request reviews/i);
    expect(src).toMatch(/status contexts/i);
  }, 30_000);
});
