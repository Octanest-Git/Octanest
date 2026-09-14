import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { DiffPatch } from "./diff-patch";

afterEach(cleanup);

describe("DiffPatch", () => {
  it("colorizes add/del/hunk lines from a unified patch", async () => {
    const patch = [
      "diff --git a/a.ts b/a.ts",
      "index 111..222 100644",
      "--- a/a.ts",
      "+++ b/a.ts",
      "@@ -1,2 +1,2 @@",
      " keep",
      "-old",
      "+new",
    ].join("\n");

    render(DiffPatch, {
      props: {
        path: "a.ts",
        status: "modified",
        patch,
      },
    });

    expect(await screen.findByText("a.ts")).toBeInTheDocument();
    expect(screen.getByText("(modified)")).toBeInTheDocument();
    const add = screen.getByText("+new");
    const del = screen.getByText("-old");
    const hunk = screen.getByText("@@ -1,2 +1,2 @@");
    expect(add.className).toMatch(/diff-add/);
    expect(del.className).toMatch(/diff-del/);
    expect(hunk.className).toMatch(/diff-hunk/);
  });
});
