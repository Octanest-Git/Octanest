import { describe, expect, it } from "vitest";
import {
  countCodeLines,
  getHighlighter,
  highlightCode,
  languageIdForPath,
  stripTrailingNewline,
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
    expect(html).toMatch(/data-language="tsrx"/);
  });

  it("highlights App.tsrx-shaped JSX and @{ statement container", async () => {
    const sample = `export function App(props: AppProps) @{
  <main>
    <h1>{props.title as string}</h1>
  </main>
}
`;
    const html = await highlightCode(sample, {
      lang: "tsrx",
      theme: "github-dark",
    });
    expect(html).toMatch(/data-language="tsrx"/);
    // Tag names are colored distinctly from punctuation (github-dark green).
    expect(html).toMatch(/color:#85E89D[^"]*">main</);
    expect(html).toMatch(/color:#85E89D[^"]*">h1</);
    // @{ statement container is a keyword-colored token.
    expect(html).toMatch(/@\{/);
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

  it("strips a trailing newline so highlight rows match line numbers", async () => {
    expect(stripTrailingNewline("a\nb\n")).toBe("a\nb");
    expect(countCodeLines("a\nb\n")).toBe(2);
    expect(countCodeLines("a\nb")).toBe(2);
    expect(countCodeLines("")).toBe(0);

    const withNl = await highlightCode("const x = 1;\n", {
      lang: "typescript",
      theme: "github-light",
    });
    const withoutNl = await highlightCode("const x = 1;", {
      lang: "typescript",
      theme: "github-light",
    });
    expect(withNl).toBe(withoutNl);
  });
});
