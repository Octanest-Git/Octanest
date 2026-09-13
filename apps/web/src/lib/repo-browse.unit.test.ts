import { describe, expect, it } from "vitest";
import { parseRefAndPath, pathBreadcrumbCrumbs } from "./repo-browse";

describe("parseRefAndPath", () => {
  it("returns empty ref and path for empty splat", () => {
    expect(parseRefAndPath("")).toEqual({ ref: "", path: "" });
    expect(parseRefAndPath(null)).toEqual({ ref: "", path: "" });
    expect(parseRefAndPath(undefined)).toEqual({ ref: "", path: "" });
  });

  it("falls back to first segment without known refs", () => {
    expect(parseRefAndPath("feature/foo/src/a.ts")).toEqual({
      ref: "feature",
      path: "foo/src/a.ts",
    });
  });

  it("falls back to first segment when no known ref matches", () => {
    expect(parseRefAndPath("feature/foo/src/a.ts", ["main", "develop"])).toEqual({
      ref: "feature",
      path: "foo/src/a.ts",
    });
  });

  it("resolves longest matching hierarchical ref prefix (WR-03 / D-17)", () => {
    const known = ["main", "feature/foo", "feature/bar"];
    expect(parseRefAndPath("feature/foo/src/a.ts", known)).toEqual({
      ref: "feature/foo",
      path: "src/a.ts",
    });
  });

  it("prefers the longest matching prefix among overlapping refs", () => {
    const known = ["feature", "feature/foo", "feature/foo/bar"];
    expect(parseRefAndPath("feature/foo/bar/x", known)).toEqual({
      ref: "feature/foo/bar",
      path: "x",
    });
  });

  it("treats whole splat as ref when it exactly matches a known ref", () => {
    expect(parseRefAndPath("feature/foo", ["feature/foo", "main"])).toEqual({
      ref: "feature/foo",
      path: "",
    });
  });
});

describe("pathBreadcrumbCrumbs", () => {
  it("returns empty crumbs for empty path", () => {
    expect(pathBreadcrumbCrumbs("")).toEqual([]);
    expect(pathBreadcrumbCrumbs("/")).toEqual([]);
  });

  it("builds prefix crumbs for a deep path (long-path chrome)", () => {
    expect(
      pathBreadcrumbCrumbs(
        "src/very/deeply/nested/components/ExtremelyLongSegmentNameThatNeedsEllipsis.tsrx",
      ),
    ).toEqual([
      { seg: "src", prefix: "src", last: false },
      { seg: "very", prefix: "src/very", last: false },
      { seg: "deeply", prefix: "src/very/deeply", last: false },
      { seg: "nested", prefix: "src/very/deeply/nested", last: false },
      {
        seg: "components",
        prefix: "src/very/deeply/nested/components",
        last: false,
      },
      {
        seg: "ExtremelyLongSegmentNameThatNeedsEllipsis.tsrx",
        prefix:
          "src/very/deeply/nested/components/ExtremelyLongSegmentNameThatNeedsEllipsis.tsrx",
        last: true,
      },
    ]);
  });
});
