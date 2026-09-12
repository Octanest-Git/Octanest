/**
 * Shiki highlighter + .tsrx/.ripple grammars (D-19).
 * RED stub: no custom langs registered so grammar tests fail intentionally.
 */
export async function getHighlighter(): Promise<{
  getLoadedLanguages: () => string[];
  getLoadedThemes: () => string[];
  codeToHtml: (code: string, options: { lang: string; theme: string }) => string;
}> {
  return {
    getLoadedLanguages: () => ["typescript", "javascript"],
    getLoadedThemes: () => ["github-light", "github-dark"],
    codeToHtml: (code, { lang }) =>
      `<pre class="shiki"><code class="language-${lang}">${code}</code></pre>`,
  };
}

export function languageIdForPath(filePath: string): string {
  const lower = filePath.toLowerCase();
  if (lower.endsWith(".tsrx")) return "typescript";
  if (lower.endsWith(".ripple")) return "javascript";
  if (lower.endsWith(".ts") || lower.endsWith(".tsx")) return "typescript";
  if (lower.endsWith(".js") || lower.endsWith(".jsx") || lower.endsWith(".mjs"))
    return "javascript";
  return "text";
}

export async function highlightCode(
  code: string,
  options: { lang: string; theme?: "github-light" | "github-dark" },
): Promise<string> {
  const highlighter = await getHighlighter();
  return highlighter.codeToHtml(code, {
    lang: options.lang,
    theme: options.theme ?? "github-dark",
  });
}
