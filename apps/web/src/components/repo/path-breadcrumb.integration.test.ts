import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { FileTree } from "./file-tree";
import { PathBreadcrumb } from "./path-breadcrumb";

afterEach(cleanup);

describe("PathBreadcrumb long-path layout (07-15 backstop)", () => {
  it("wraps deep paths and truncates long segments with title", () => {
    const longSeg = "ExtremelyLongSegmentNameThatNeedsEllipsisAndMustNotBlowLayout.tsrx";
    const deep = `src/a/b/c/d/e/${longSeg}`;

    const { container } = render(PathBreadcrumb, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        path: deep,
      },
    });

    const nav = screen.getByRole("navigation", { name: "Path" });
    expect(nav.className).toMatch(/flex-wrap/);
    expect(nav.className).toMatch(/min-w-0/);
    expect(nav.className).toMatch(/max-w-full/);

    const last = screen.getByTitle(longSeg);
    expect(last).toBeInTheDocument();
    expect(last.className).toMatch(/truncate/);
    expect(last).toHaveTextContent(longSeg);

    // Intermediate crumbs remain links with title for hover full name
    expect(screen.getByRole("link", { name: "src" })).toHaveAttribute("title", "src");

    // No single-line forced overflow class on the nav itself
    expect(nav.className).not.toMatch(/whitespace-nowrap/);
    expect(container.querySelectorAll("a").length).toBeGreaterThan(2);
  });
});

describe("FileTree long name layout (07-15 / E3 overflow)", () => {
  it("truncates long entry names and exposes full name via title", () => {
    const longName = "a-very-very-long-filename-that-should-ellipsis-in-the-tree-row.ts";
    render(FileTree, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        basePath: "src",
        entries: [
          {
            name: longName,
            kind: "blob",
            oid: "abc1234deadbeef",
            mode: "100644",
          },
        ],
      },
    });

    const link = screen.getByRole("link", { name: longName });
    expect(link.className).toMatch(/truncate/);
    expect(link.className).toMatch(/min-w-0/);
    expect(link).toHaveAttribute("title", longName);
  });
});

describe("FileTree parent row (GitHub/Gitea ..)", () => {
  it("shows .. linking to parent when basePath is nested", () => {
    render(FileTree, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        basePath: "src/lib",
        entries: [
          {
            name: "index.ts",
            kind: "blob",
            oid: "abc1234deadbeef",
            mode: "100644",
          },
        ],
      },
    });

    const parent = screen.getByRole("link", { name: ".." });
    expect(parent).toHaveAttribute("href", "/ada/hello/tree/main/src");
    expect(parent).toHaveAttribute("title", "Parent directory");
  });

  it("omits .. at repo root", () => {
    render(FileTree, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        basePath: "",
        entries: [
          {
            name: "README.md",
            kind: "blob",
            oid: "abc1234deadbeef",
            mode: "100644",
          },
        ],
      },
    });

    expect(screen.queryByRole("link", { name: ".." })).not.toBeInTheDocument();
  });
});

describe("PathBreadcrumb repo crumb", () => {
  it("links the repo name to code home", () => {
    render(PathBreadcrumb, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        path: "src",
      },
    });

    expect(screen.getByRole("link", { name: "hello" })).toHaveAttribute("href", "/ada/hello/");
  });
});
