import { describe, expect, it } from "vitest";

/**
 * ORG-03 / D-ORG-02c / D-ORG-04 Wave 0 stubs: repo settings Collaborators panel.
 *
 * RED until collaborators panel + can_admin gate land (10-11).
 * Do not implement production Collaborators UI here.
 *
 * Uses a variable dynamic import + @vite-ignore so Vitest can collect the
 * suite while the panel module is still absent.
 */

async function loadCollaboratorsPanel(): Promise<Record<string, unknown>> {
  const rel = "../components/repo/collaborators-panel";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: Collaborators panel missing — implement in 10-11 (ORG-03 / D-ORG-02c / D-ORG-04). Expected Collaborators section gated by can_admin + Add collaborator + read|write|admin ladder. ${(err as Error).message}`,
    );
  }
}

describe("repo settings Collaborators Wave 0 (ORG-03 / D-ORG-02c / D-ORG-04)", () => {
  it("settings gate uses can_admin — not me.id === owner_id", async () => {
    const panel = await loadCollaboratorsPanel();
    // Greened in 10-11: show settings/Collaborators when repo.can_admin
    expect(
      panel.CollaboratorsPanel ?? panel.default,
      "Wave 0: CollaboratorsPanel must gate on can_admin (D-ORG-04)",
    ).toBeTruthy();
  });

  it("Collaborators section: list + Add collaborator empty state", async () => {
    const panel = await loadCollaboratorsPanel();
    // Greened in 10-11: heading Collaborators; empty short sentence + Add collaborator
    expect(
      panel.CollaboratorsPanel ?? panel.default,
      "Wave 0: CollaboratorsPanel export for list/add empty state",
    ).toBeTruthy();
  });

  it("add/update/remove permission ladder read | write | admin", async () => {
    const panel = await loadCollaboratorsPanel();
    // Greened in 10-11: Permission Select read|write|admin; works personal + org-owned
    expect(
      panel.CollaboratorsPanel ?? panel.default,
      "Wave 0: permission ladder read|write|admin on add/update (D-ORG-02c)",
    ).toBeTruthy();
  });

  it("username lookup autocomplete never shows email (T-10-03)", async () => {
    const panel = await loadCollaboratorsPanel();
    // Greened in 10-11: user.lookup results render username/display/avatar only
    expect(
      panel.CollaboratorsPanel ?? panel.default,
      "Wave 0: collaborator lookup must never render email addresses (T-10-03 / D-ORG-03)",
    ).toBeTruthy();
  });
});
