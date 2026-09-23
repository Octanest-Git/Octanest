import { describe, expect, it } from "@octanest/web/test-runner";
import {
  ORG_PROFILE_README_DIR,
  ORG_PROFILE_SPECIAL_REPOS,
  pickOrgProfileSpecialRepo,
} from "./profile-readme";

describe("profile-readme resolution", () => {
  it("prefers .octanest over .github when both are public", () => {
    expect(
      pickOrgProfileSpecialRepo([
        { name: ".github", visibility: "public" },
        { name: ".octanest", visibility: "public" },
      ]),
    ).toBe(".octanest");
  });

  it("falls back to .github when .octanest is missing", () => {
    expect(pickOrgProfileSpecialRepo([{ name: ".github", visibility: "public" }])).toBe(".github");
  });

  it("ignores private special repos (public profile only)", () => {
    expect(
      pickOrgProfileSpecialRepo([
        { name: ".octanest", visibility: "private" },
        { name: ".github", visibility: "private" },
      ]),
    ).toBeNull();

    expect(
      pickOrgProfileSpecialRepo([
        { name: ".octanest", visibility: "private" },
        { name: ".github", visibility: "public" },
      ]),
    ).toBe(".github");
  });

  it("returns null when no special repos exist", () => {
    expect(pickOrgProfileSpecialRepo([{ name: "demo", visibility: "public" }])).toBeNull();
    expect(pickOrgProfileSpecialRepo([])).toBeNull();
  });

  it("documents preference order and org README directory", () => {
    expect([...ORG_PROFILE_SPECIAL_REPOS]).toEqual([".octanest", ".github"]);
    expect(ORG_PROFILE_README_DIR).toBe("profile");
  });
});
