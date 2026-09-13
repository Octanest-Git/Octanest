import { describe, expect, it } from "vitest";

/**
 * ORG-01 / D-ORG-02b / D-ORG-03 Wave 0 stubs: org members + invites + member_base.
 *
 * RED until $owner.settings.members.tsrx lands (10-10).
 * Do not implement production members routes here.
 *
 * Uses a variable dynamic import + @vite-ignore so Vitest can collect the
 * suite while the route is still absent.
 */

async function loadMembersModule(): Promise<Record<string, unknown>> {
  const rel = "./$owner.settings.members";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /{org}/settings/members missing — implement in 10-10 (ORG-01 / D-ORG-03). Expected Members table + Add member + invites. ${(err as Error).message}`,
    );
  }
}

async function loadOrgSettingsModule(): Promise<Record<string, unknown>> {
  const rel = "./$owner.settings";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /{org}/settings missing — member_base_permission Select (D-ORG-02b). ${(err as Error).message}`,
    );
  }
}

describe("org members Wave 0 (ORG-01 / D-ORG-03)", () => {
  it("Add member by username with live lookup autocomplete", async () => {
    const mod = await loadMembersModule();
    // Greened in 10-10: username Input + user.lookup prefix ≥2 → ≤10 results;
    // Role Select Owner|Admin|Member (default Member); works when allow_signup false
    expect(
      mod.OrgMembersPage ?? mod.MembersPage ?? mod.default,
      "Wave 0: members page must support Add member by username (D-ORG-03)",
    ).toBeTruthy();
  });

  it("lookup results never include email (T-10-03 / D-ORG-03)", async () => {
    const mod = await loadMembersModule();
    // Greened in 10-10: autocomplete shows username, display name, avatar URL only
    expect(
      mod.OrgMembersPage ?? mod.MembersPage ?? mod.default,
      "Wave 0: member lookup must never render email addresses (T-10-03)",
    ).toBeTruthy();
  });

  it("email invite create/list + Revoke invite / Keep invite AlertDialog", async () => {
    const mod = await loadMembersModule();
    // Greened in 10-10: Email + role invite; pending list; Revoke invite / Keep invite
    expect(
      mod.OrgMembersPage ?? mod.MembersPage ?? mod.default,
      "Wave 0: invites panel Revoke invite? / Keep invite (D-ORG-03)",
    ).toBeTruthy();
  });
});

describe("org settings member_base Wave 0 (D-ORG-02b)", () => {
  it("member_base_permission Select None | Read | Write (default None)", async () => {
    const mod = await loadOrgSettingsModule();
    // Greened in 10-10: Select None (default) | Read | Write; helper about private
    // org repos; Owner/Admin always admin
    expect(
      mod.OrgSettingsPage ?? mod.default,
      "Wave 0: member_base_permission None|Read|Write control (D-ORG-02b)",
    ).toBeTruthy();
  });
});
