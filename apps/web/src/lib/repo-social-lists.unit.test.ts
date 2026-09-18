import { describe, expect, it } from "vitest";
import { forksSortLabel, parseForksSort, socialListHref } from "@/lib/repo-social-lists";

describe("repo-social-lists helpers", () => {
  it("parses forks sort aliases", () => {
    expect(parseForksSort("stars")).toBe("stars");
    expect(parseForksSort("updated")).toBe("updated");
    expect(parseForksSort("recently_updated")).toBe("updated");
    expect(parseForksSort("created")).toBe("created");
    expect(parseForksSort("bogus")).toBe("stars");
    expect(parseForksSort(null)).toBe("stars");
  });

  it("labels forks sort options", () => {
    expect(forksSortLabel("stars")).toBe("Most starred");
    expect(forksSortLabel("updated")).toBe("Recently updated");
    expect(forksSortLabel("created")).toBe("Recently created");
  });

  it("builds social list hrefs", () => {
    expect(socialListHref("/o/r/forks")).toBe("/o/r/forks");
    expect(socialListHref("/o/r/forks", { q: "alice" })).toBe("/o/r/forks?q=alice");
    expect(socialListHref("/o/r/forks", { sort: "updated" })).toBe("/o/r/forks?sort=updated");
    expect(socialListHref("/o/r/forks", { sort: "stars", q: "x" })).toBe("/o/r/forks?q=x");
  });
});
