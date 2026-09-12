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
});
