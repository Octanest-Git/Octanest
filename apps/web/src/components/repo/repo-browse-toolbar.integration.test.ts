import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { RepoBrowseToolbar } from "./repo-browse-toolbar";

afterEach(cleanup);

describe("RepoBrowseToolbar", () => {
  it("shows path crumbs and clone control on tree pages", () => {
    render(RepoBrowseToolbar, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        refs: [
          {
            name: "refs/heads/main",
            oid: "abc1234deadbeef",
            kind: "branch",
          },
        ],
        path: "src/lib",
        showPathCrumbs: true,
        empty: false,
      },
    });

    expect(screen.getByTestId("repo-browse-toolbar")).toBeInTheDocument();
    expect(screen.getByRole("navigation", { name: "Path" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "hello" })).toHaveAttribute("href", "/ada/hello/");
    expect(screen.getByRole("link", { name: "src" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clone or download" })).toBeInTheDocument();
  });

  it("omits path crumbs on code home", () => {
    render(RepoBrowseToolbar, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        refs: [
          {
            name: "refs/heads/main",
            oid: "abc1234deadbeef",
            kind: "branch",
          },
        ],
        showPathCrumbs: false,
        empty: false,
      },
    });

    expect(screen.queryByRole("navigation", { name: "Path" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clone or download" })).toBeInTheDocument();
  });
});
