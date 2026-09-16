import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Phase 12 Pulls UI Wave 0 stubs — greened in plans 03–07.
 * Prefer it.fails so the suite stays runnable while cases remain RED.
 */

const chromeActive = readFileSync(
  join(process.cwd(), "src/lib/repo-chrome-active.ts"),
  "utf8",
);
const repoChrome = readFileSync(
  join(process.cwd(), "src/components/repo/repo-chrome.tsrx"),
  "utf8",
);

describe("Phase 12 Pulls UI (Wave 0)", () => {
  it.fails("RepoChromeActive includes pulls and maps /pulls|/pull", () => {
    expect(chromeActive).toMatch(/"pulls"/);
    expect(chromeActive).toMatch(/pulls:\s*"pulls"/);
  });

  it.fails("RepoChrome renders Pulls tab", () => {
    expect(repoChrome).toMatch(/Pulls/);
    expect(repoChrome).toMatch(/\/pulls/);
  });

  it.fails("pulls list route defaults Open with Closed/All", () => {
    expect(true).toBe(false);
  });

  it.fails("New pull request gated by can_write", () => {
    expect(true).toBe(false);
  });

  it.fails("compare flow can create a PR", () => {
    expect(true).toBe(false);
  });

  it.fails("detail tabs Conversation | Commits | Files changed", () => {
    expect(true).toBe(false);
  });

  it.fails("unified and split diff toggle", () => {
    expect(true).toBe(false);
  });

  it.fails("review actions Approve / Request changes / Comment", () => {
    expect(true).toBe(false);
  });

  it.fails("merge method picker + close/reopen", () => {
    expect(true).toBe(false);
  });

  it.fails("Admin merge strategy settings", () => {
    expect(true).toBe(false);
  });

  it.fails("Write|Preview on PR comments", () => {
    expect(true).toBe(false);
  });
});
