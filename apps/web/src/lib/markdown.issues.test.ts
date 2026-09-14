import { describe, expect, it } from "vitest";
import { renderGfm } from "./markdown";

/**
 * Phase 11 ISS-04 / D-ISS-13 / D-ISS-10 Wave 0 stubs for issue autolink.
 *
 * Autolink greens in plan 11-10 (remark-github + buildUrl). Do not install
 * remark-github in this Wave 0 plan. Prefer `it.fails` for still-RED cases so
 * the unit suite stays green while stubs remain discoverable.
 *
 * Q1 RESOLVED: mention/commit autolinks disabled via buildUrl → false.
 */

type RenderGfmWithRepo = (
  markdown: string,
  opts?: { owner?: string; repo?: string },
) => Promise<string>;

const renderWithRepo = renderGfm as RenderGfmWithRepo;

describe("markdown.issues Wave 0 (ISS-04 / D-ISS-13)", () => {
  it.fails(
    "turns #N into /{owner}/{repo}/issues/{n} href (ISS-04 / D-ISS-13)",
    async () => {
      const html = await renderWithRepo("See #42 for details", {
        owner: "ada",
        repo: "hello",
      });
      expect(html).toMatch(/href=["']\/ada\/hello\/issues\/42["']/);
      expect(html).toMatch(/>#42</);
    },
  );

  it.fails(
    "turns owner/repo#N into /owner/repo/issues/n href (ISS-04 / D-ISS-13)",
    async () => {
      const html = await renderWithRepo("Related: bob/world#7", {
        owner: "ada",
        repo: "hello",
      });
      expect(html).toMatch(/href=["']\/bob\/world\/issues\/7["']/);
    },
  );

  it("keeps rehype-sanitize last — script bodies neutralized (D-ISS-10 / T-11-03)", async () => {
    const html = await renderGfm(
      'Hello <script>alert("xss")</script> **world** see #1',
    );
    expect(html).not.toMatch(/<script/i);
    expect(html).not.toMatch(/\son\w+=/i);
    expect(html).toMatch(/<strong>world<\/strong>/i);
  });

  it("does not autolink @mentions (buildUrl false / Q1)", async () => {
    const html = await renderWithRepo("Thanks @ada for the review", {
      owner: "ada",
      repo: "hello",
    });
    expect(html).not.toMatch(/href=["'][^"']*\/ada["']/);
    expect(html).toMatch(/@ada/);
  });

  it("does not autolink commit SHAs (buildUrl false / Q1)", async () => {
    const html = await renderWithRepo(
      "Landed in abcdef0123456789abcdef0123456789abcdef01",
      { owner: "ada", repo: "hello" },
    );
    expect(html).not.toMatch(/href=["'][^"']*\/commit\//);
    expect(html).not.toMatch(/href=["'][^"']*\/commits\//);
  });
});
