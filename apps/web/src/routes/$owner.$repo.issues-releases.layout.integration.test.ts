/**
 * Layout parents for issues/releases must Outlet so detail/new routes render.
 * Leaf chrome lives only on `$owner.$repo` (D-QH-01).
 */
import { describe, expect, it } from "vitest";

describe("issues/releases layout parents", () => {
  it("issues layout Outlets; list is on index", async () => {
    const layout = await import("./$owner.$repo.issues.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    const index = await import("./$owner.$repo.issues.index.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(layout).toMatch(/Outlet/);
    expect(layout).not.toMatch(/IssuesListPage/);
    expect(index).toMatch(/IssuesListPage/);
  });

  it("releases layout Outlets; list is on index + SSR release.list", async () => {
    const layout = await import("./$owner.$repo.releases.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    const index = await import("./$owner.$repo.releases.index.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(layout).toMatch(/Outlet/);
    expect(layout).not.toMatch(/RepoReleasesPage/);
    expect(index).toMatch(/RepoReleasesPage/);
    expect(index).toMatch(/fetchReleaseList/);
  });

  it("issues/releases/settings leaves inherit layout chrome — no leaf RepoChrome (D-QH-01)", async () => {
    const sources = await Promise.all([
      import("./$owner.$repo.issues.index.tsrx?raw"),
      import("./$owner.$repo.issues.new.tsrx?raw"),
      import("./$owner.$repo.issues.$n.tsrx?raw"),
      import("./$owner.$repo.issues.labels.tsrx?raw"),
      import("./$owner.$repo.releases.index.tsrx?raw"),
      import("./$owner.$repo.releases.new.tsrx?raw"),
      import("./$owner.$repo.releases.$tag.tsrx?raw"),
      import("./$owner.$repo.settings.tsrx?raw"),
    ]);
    const names = [
      "issues.index",
      "issues.new",
      "issues.$n",
      "issues.labels",
      "releases.index",
      "releases.new",
      "releases.$tag",
      "settings",
    ];
    for (let i = 0; i < sources.length; i++) {
      const src = String((sources[i] as { default: string }).default);
      expect(src, `${names[i]} must not remount RepoChrome`).not.toMatch(/RepoChrome/);
    }
    const repoLayout = await import("./$owner.$repo.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(repoLayout).toMatch(/RepoLayoutChrome/);
    expect(repoLayout).toMatch(/RepoChrome/);
    expect(repoLayout).toMatch(/repoChromeActiveFromPath/);
  }, 30_000);
});
