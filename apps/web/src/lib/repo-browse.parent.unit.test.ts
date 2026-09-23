import { describe, expect, it } from "@octanest/web/test-runner";
import { parentRepoPath, treeHref } from "./repo-browse";

describe("parentRepoPath", () => {
  it("returns empty at repo root and single segment", () => {
    expect(parentRepoPath("")).toBe("");
    expect(parentRepoPath("src")).toBe("");
    expect(parentRepoPath("/src/")).toBe("");
  });

  it("strips the last segment for nested paths", () => {
    expect(parentRepoPath("src/lib")).toBe("src");
    expect(parentRepoPath("a/b/c")).toBe("a/b");
    expect(parentRepoPath("/packages/web/src/")).toBe("packages/web");
  });
});

describe("treeHref parent navigation", () => {
  it("maps nested parent to tree URL under the same ref", () => {
    expect(treeHref("ada", "hello", "main", parentRepoPath("src/lib"))).toBe(
      "/ada/hello/tree/main/src",
    );
    expect(treeHref("ada", "hello", "main", parentRepoPath("src"))).toBe("/ada/hello/tree/main");
  });
});
