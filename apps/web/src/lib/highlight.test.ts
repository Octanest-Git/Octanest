import { describe, expect, it } from "vitest";
import {
  getHighlighter,
  highlightCode,
  languageIdForPath,
} from "./highlight";

describe("highlight", () => {
  it("maps .tsrx and .ripple via in-repo grammars not TS/JS alias alone", async () => {
    expect(languageIdForPath("App.tsrx")).toBe("tsrx");
    expect(languageIdForPath("view.ripple")).toBe("ripple");

    const highlighter = await getHighlighter();
    const langs = highlighter.getLoadedLanguages();
    expect(langs).toContain("tsrx");
    expect(langs).toContain("ripple");

    const themes = highlighter.getLoadedThemes();
    expect(themes).toContain("github-light");
    expect(themes).toContain("github-dark");
  });

  it("highlights tsrx source with registered language id", async () => {
    const html = await highlightCode('@if (true) { "ok" }', {
      lang: "tsrx",
      theme: "github-dark",
    });
    expect(html).toMatch(/shiki/i);
    expect(html).toMatch(/tsrx|source\.tsrx/i);
  });

  it("maps and highlights .ts blobs as typescript (07-15 UAT)", async () => {
    expect(languageIdForPath("src/util.ts")).toBe("typescript");
    const html = await highlightCode("const x: number = 1;", {
      lang: languageIdForPath("src/util.ts"),
      theme: "github-dark",
    });
    expect(html).toMatch(/shiki/i);
    expect(html).toMatch(/language-typescript|typescript/i);
    expect(html).toMatch(/const/);
  });
});
