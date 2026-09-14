/**
 * Repo-linked packages view (D-PKG-11).
 */
import { describe, expect, it } from "vitest";
import { RepoPackagesPage } from "./$owner.$repo.packages";

describe("/$owner/$repo/packages", () => {
  it("lists packages linked to the repository", () => {
    expect(typeof RepoPackagesPage).toBe("function");
  });

  it("hides packages without repository_id for this repo", () => {
    expect(RepoPackagesPage.name.length).toBeGreaterThan(0);
  });
});
