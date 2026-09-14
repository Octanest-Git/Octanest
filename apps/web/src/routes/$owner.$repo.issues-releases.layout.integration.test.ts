/**
 * Layout parents for issues/releases must Outlet so detail/new routes render.
 */
import { describe, expect, it } from "vitest";

describe("issues/releases layout parents", () => {
  it("issues layout Outlets; list is on index", async () => {
    const layout = await import("./$owner.$repo.issues.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    const index = await import("./$owner.$repo.issues.index.tsrx?raw").then(
      (m) => String((m as { default: string }).default),
    );
    expect(layout).toMatch(/Outlet/);
    expect(layout).not.toMatch(/IssuesListPage/);
    expect(index).toMatch(/IssuesListPage/);
  });

  it("releases layout Outlets; list is on index + SSR release.list", async () => {
    const layout = await import("./$owner.$repo.releases.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    const index = await import("./$owner.$repo.releases.index.tsrx?raw").then(
      (m) => String((m as { default: string }).default),
    );
    expect(layout).toMatch(/Outlet/);
    expect(layout).not.toMatch(/RepoReleasesPage/);
    expect(index).toMatch(/RepoReleasesPage/);
    expect(index).toMatch(/fetchReleaseList/);
  });
});
