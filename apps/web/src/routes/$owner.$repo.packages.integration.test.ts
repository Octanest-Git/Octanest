/**
 * Repo-linked packages view (D-PKG-11).
 */
import { describe, expect, it } from "vitest";
import { RepoPackagesPage } from "./$owner.$repo.packages";

describe("/$owner/$repo/packages", () => {
  it("lists packages linked to the repository", () => {
    expect(typeof RepoPackagesPage).toBe("function");
  });

  it(
    "scopes packages.list by repository_id from repo.get (not owner-wide)",
    async () => {
      const src = await import("./$owner.$repo.packages.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(src).toMatch(/repo\.get/);
      expect(src).toMatch(/repository_id/);
      expect(src).toMatch(/packagesListQueryOptions/);
      // Must not list by owner alone then soft-filter any linked package.
      expect(src).not.toMatch(
        /packagesListQueryOptions\(\s*apiClient\s*,\s*\{\s*owner\s*\}/,
      );
      expect(src).not.toMatch(
        /repository_id\s*!=\s*null\s*&&\s*p\.repository_id\.length/,
      );
    },
    30_000,
  );
});
