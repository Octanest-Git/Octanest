import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const dir = dirname(fileURLToPath(import.meta.url));

describe("$owner user profile (SOC-02)", () => {
  it("branches user vs org on owner index", () => {
    const src = readFileSync(join(dir, "$owner.index.tsrx"), "utf8");
    expect(src).toMatch(/export function UserProfilePage/);
    expect(src).toMatch(/export function OrgOverviewPage/);
    expect(src).toMatch(/fetchUserProfile/);
    expect(src).toMatch(/fetchOrgOverview/);
  });

  it("wires profile README panel on user and org overview", () => {
    const src = readFileSync(join(dir, "$owner.index.tsrx"), "utf8");
    expect(src).toMatch(/ReadmePanel/);
    expect(src).toMatch(/profileReadme/);

    const userSsr = readFileSync(join(dir, "../lib/ssr-user-profile.ts"), "utf8");
    expect(userSsr).toMatch(/fetchUserProfileReadme/);
    expect(userSsr).toMatch(/profileReadme/);

    const orgSsr = readFileSync(join(dir, "../lib/ssr-org.ts"), "utf8");
    expect(orgSsr).toMatch(/fetchOrgProfileReadme/);
    expect(orgSsr).toMatch(/profileReadme/);
  });
});
