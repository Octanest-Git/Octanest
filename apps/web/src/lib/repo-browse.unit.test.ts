import { describe, expect, it } from "vitest";
import type { RepoTreeEntry } from "@octanest/api-client";
import {
  activityHref,
  blobHref,
  findContributingName,
  findLicenseName,
  formatFileSize,
  licenseSidebarLabel,
  parseRefAndPath,
  pathBreadcrumbCrumbs,
  resolveAboutRootFiles,
} from "./repo-browse";
import { languagePercent } from "./language-bar";

function blob(name: string): RepoTreeEntry {
  return { name, kind: "blob", oid: "abc", mode: "100644" };
}

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
        prefix: "src/very/deeply/nested/components/ExtremelyLongSegmentNameThatNeedsEllipsis.tsrx",
        last: true,
      },
    ]);
  });
});

describe("formatFileSize", () => {
  it("formats bytes / KB / MB", () => {
    expect(formatFileSize(0)).toBe("0 Bytes");
    expect(formatFileSize(58)).toBe("58 Bytes");
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(10_240)).toBe("10 KB");
    expect(formatFileSize(2_097_152)).toBe("2 MB");
  });
});

describe("findLicenseName / findContributingName", () => {
  it("prefers LICENSE then COPYING-style names", () => {
    expect(findLicenseName([blob("README.md"), blob("LICENSE")])).toBe("LICENSE");
    expect(findLicenseName([blob("license.md")])).toBe("license.md");
    expect(findLicenseName([blob("COPYING")])).toBe("COPYING");
    expect(findLicenseName([blob("src")])).toBeNull();
  });

  it("finds CONTRIBUTING.md case-insensitively", () => {
    expect(findContributingName([blob("CONTRIBUTING.md")])).toBe("CONTRIBUTING.md");
    expect(findContributingName([blob("contributing")])).toBe("contributing");
    expect(findContributingName([blob("README.md")])).toBeNull();
  });
});

describe("licenseSidebarLabel", () => {
  it("reads SPDX stub and common first lines", () => {
    expect(licenseSidebarLabel("LICENSE", "SPDX-License-Identifier: MIT\n")).toBe("MIT license");
    expect(licenseSidebarLabel("LICENSE", "MIT License\n\nCopyright")).toBe("MIT license");
    expect(licenseSidebarLabel("LICENSE", "Apache License\n")).toBe("Apache-2.0 license");
  });

  it("falls back to filename heuristic then License", () => {
    expect(licenseSidebarLabel("LICENSE-MIT")).toBe("MIT license");
    expect(licenseSidebarLabel("LICENSE")).toBe("License");
  });
});

describe("resolveAboutRootFiles", () => {
  it("resolves license label from fetched text and finds contributing", async () => {
    const meta = await resolveAboutRootFiles(
      [blob("LICENSE"), blob("CONTRIBUTING.md"), blob("README.md")],
      async (path) => (path === "LICENSE" ? "MIT License\n" : null),
    );
    expect(meta.licenseFile).toBe("LICENSE");
    expect(meta.licenseLabel).toBe("MIT license");
    expect(meta.contributingFile).toBe("CONTRIBUTING.md");
  });
});

describe("activityHref / blobHref About links", () => {
  it("builds GitHub-shaped activity and license blob URLs", () => {
    expect(activityHref("octanest", "forge")).toBe("/octanest/forge/activity");
    expect(blobHref("octanest", "forge", "main", "LICENSE")).toBe(
      "/octanest/forge/blob/main/LICENSE",
    );
    expect(blobHref("octanest", "forge", "feat/x", "CONTRIBUTING.md")).toBe(
      "/octanest/forge/blob/feat%2Fx/CONTRIBUTING.md",
    );
  });
});

describe("languagePercent", () => {
  it("formats GitHub-style one-decimal percentages", () => {
    expect(languagePercent(950, 1000)).toBe("95.0%");
    expect(languagePercent(1, 10000)).toBe("<0.1%");
    expect(languagePercent(0, 100)).toBe("0%");
  });
});
