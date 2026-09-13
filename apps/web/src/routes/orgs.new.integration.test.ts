import { describe, expect, it } from "vitest";

/**
 * ORG-01 / D-ORG-01 / D-ORG-06 Wave 0 stubs: /orgs/new create UI.
 *
 * RED until orgs.new.tsrx lands (10-13).
 * Do not implement production routes here.
 *
 * Uses a variable dynamic import + @vite-ignore so Vitest can collect the
 * suite while ./orgs.new is still absent (static path fails transform).
 */

/** Load /orgs/new page; path is runtime-only so Vite does not resolve at transform. */
async function loadOrgsNewModule(): Promise<Record<string, unknown>> {
  const rel = "./orgs.new";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /orgs/new route missing — implement in 10-13 (ORG-01 / D-ORG-01 / D-ORG-06). Expected title New organization · Octanest + Create organization CTA. ${(err as Error).message}`,
    );
  }
}

describe("/orgs/new Wave 0 (ORG-01 / D-ORG-01 / D-ORG-06)", () => {
  it("verified: Slug + optional Display name + Create organization CTA", async () => {
    const mod = await loadOrgsNewModule();
    // Greened in 10-13: verified session → form with Slug (required), Display name
    // (optional), Create organization; success navigates to /{slug}
    expect(
      mod.OrgsNewPage ?? mod.NewOrgPage ?? mod.default,
      "Wave 0: orgs.new must export OrgsNewPage for Create organization form",
    ).toBeTruthy();
  });

  it("unverified: Verify your email wall — not the create form", async () => {
    const mod = await loadOrgsNewModule();
    // Greened in 10-13: same verify wall pattern as /new
    expect(
      mod.OrgsNewPage ?? mod.NewOrgPage ?? mod.default,
      "Wave 0: unverified must show Verify your email wall (not Slug form)",
    ).toBeTruthy();
  });

  it("reserved slug shows That username is reserved. Choose a different username.", async () => {
    const mod = await loadOrgsNewModule();
    // Greened in 10-13: auth.reserved_username → same copy as signup RESERVED_ERROR
    expect(
      mod.OrgsNewPage ?? mod.NewOrgPage ?? mod.default,
      "Wave 0: reserved slug must map to That username is reserved. Choose a different username.",
    ).toBeTruthy();
  });

  it("taken slug shows slug already used by a user or org", async () => {
    const mod = await loadOrgsNewModule();
    // Greened in 10-13: dual uniqueness vs users+orgs (D-ORG-01 shared namespace)
    expect(
      mod.OrgsNewPage ?? mod.NewOrgPage ?? mod.default,
      "Wave 0: taken slug error must explain slug already used by a user or org",
    ).toBeTruthy();
  });
});
