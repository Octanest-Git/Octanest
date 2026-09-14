import { describe, expect, it } from "vitest";

/**
 * Wave 0 / Phase 15: GIT-14/15 Releases tab + routes (D-REL-13).
 * Turned green in 15-06 (notes) / 15-02 (assets).
 */

describe("repo Releases tab (GIT-14/15 / D-REL-13)", () => {
  it(
    "Releases tab present in repo chrome",
    async () => {
      const chromeSrc = await import("../components/repo/repo-chrome.tsrx?raw").then(
        (m) => String((m as { default: string }).default),
      );
      expect(
        chromeSrc,
        "Wave 0: repo-chrome must include Releases tab (D-REL-13)",
      ).toMatch(/Releases/);
      expect(chromeSrc).toMatch(/releases/);
    },
    30_000,
  );

  it(
    "routes under /{owner}/{repo}/releases discoverable",
    async () => {
      const list = await import("./$owner.$repo.releases.index").catch(() => null);
      const create = await import("./$owner.$repo.releases.new").catch(() => null);
      const detail = await import("./$owner.$repo.releases.$tag").catch(() => null);
      expect(
        list?.RepoReleasesPage ?? list?.default ?? list,
        "Wave 0: $owner.$repo.releases route must export (15-06)",
      ).toBeTruthy();
      expect(
        create?.RepoReleaseNewPage ?? create?.default ?? create,
        "Wave 0: $owner.$repo.releases.new route must export (15-06)",
      ).toBeTruthy();
      expect(
        detail?.RepoReleaseDetailPage ?? detail?.default ?? detail,
        "Wave 0: $owner.$repo.releases.$tag route must export (15-06)",
      ).toBeTruthy();
    },
    30_000,
  );
});
