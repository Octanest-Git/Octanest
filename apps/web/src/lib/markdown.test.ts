import { describe, expect, it } from "vitest";
import { renderGfm } from "./markdown";

describe("renderGfm", () => {
  it("strips script tags and unsafe HTML", async () => {
    const html = await renderGfm('Hello <script>alert("xss")</script> **world**');
    // Tags/handlers must go; inert text left after strip is not executable XSS.
    expect(html).not.toMatch(/<script/i);
    expect(html).not.toMatch(/\son\w+=/i);
    expect(html).toMatch(/<strong>world<\/strong>/i);
  });

  it("renders GFM tables without raw HTML passthrough", async () => {
    const html = await renderGfm("| a | b |\n| --- | --- |\n| 1 | <img onerror=alert(1) src=x> |");
    expect(html).toMatch(/<table/i);
    expect(html).not.toMatch(/onerror/i);
  });
});
