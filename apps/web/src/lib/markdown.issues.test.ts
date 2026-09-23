import { describe, expect, it } from "@octanest/web/test-runner";
import { renderGfm } from "./markdown";

/**
 * Phase 11 ISS-04 / D-ISS-13 / D-ISS-10 — issue autolink via remark-github.
 *
 * Q1 RESOLVED: mention/commit autolinks disabled via buildUrl → false.
 */

describe("markdown.issues (ISS-04 / D-ISS-13)", () => {
  it("turns #N into /{owner}/{repo}/issues/{n} href (ISS-04 / D-ISS-13)", async () => {
    const html = await renderGfm("See #42 for details", {
      owner: "ada",
      repo: "hello",
    });
    expect(html).toMatch(/href=["']\/ada\/hello\/issues\/42["']/);
    expect(html).toMatch(/>#42</);
  });

  it("turns owner/repo#N into /owner/repo/issues/n href (ISS-04 / D-ISS-13)", async () => {
    const html = await renderGfm("Related: bob/world#7", {
      owner: "ada",
      repo: "hello",
    });
    expect(html).toMatch(/href=["']\/bob\/world\/issues\/7["']/);
  });

  it("keeps rehype-sanitize last — script bodies neutralized (D-ISS-10 / T-11-03)", async () => {
    const html = await renderGfm('Hello <script>alert("xss")</script> **world** see #1');
    expect(html).not.toMatch(/<script/i);
    expect(html).not.toMatch(/\son\w+=/i);
    expect(html).toMatch(/<strong>world<\/strong>/i);
  });

  it("does not autolink @mentions (buildUrl false / Q1)", async () => {
    const html = await renderGfm("Thanks @ada for the review", {
      owner: "ada",
      repo: "hello",
    });
    expect(html).not.toMatch(/href=["'][^"']*\/ada["']/);
    expect(html).toMatch(/@ada/);
  });

  it("does not autolink commit SHAs (buildUrl false / Q1)", async () => {
    const html = await renderGfm("Landed in abcdef0123456789abcdef0123456789abcdef01", {
      owner: "ada",
      repo: "hello",
    });
    expect(html).not.toMatch(/href=["'][^"']*\/commit\//);
    expect(html).not.toMatch(/href=["'][^"']*\/commits\//);
  });
});
