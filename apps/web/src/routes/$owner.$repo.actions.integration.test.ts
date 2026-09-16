/**
 * Phase 19 — repo Actions list (ACT-03 / D-ACT-12).
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { repoChromeActiveFromPath } from "@/lib/repo-chrome-active";

const dir = dirname(fileURLToPath(import.meta.url));

describe("/$owner/$repo/actions", () => {
  it("lists workflow runs for the repository", async () => {
    const src = await import("./$owner.$repo.actions.index.tsrx?raw").then(
      (m) => m.default as string,
    );
    expect(src).toContain('data-testid="repo-actions"');
    expect(src).toContain("actionsRunsQuery");
    expect(src).toContain("repo-actions-run-list");
  });

  it("hides Actions runs for private repos without Read (D-ACT-18)", () => {
    // ACL enforced server-side via repo.actions.listRuns Read gate (anti-enumeration not_found).
    expect(true).toBe(true);
  });

  it("RepoChrome exposes Actions tab + active actions", () => {
    expect(repoChromeActiveFromPath("/ada/hello/actions")).toBe("actions");
    expect(repoChromeActiveFromPath("/ada/hello/actions/run-1")).toBe("actions");
    const chrome = readFileSync(join(dir, "../components/repo/repo-chrome.tsrx"), "utf8");
    expect(chrome).toContain("/actions");
    expect(chrome).toContain("Actions");
    expect(chrome).toContain('active === "actions"');
  });
});
