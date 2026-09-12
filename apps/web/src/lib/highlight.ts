import { createHighlighter, type Highlighter } from "shiki";
import tsrxGrammar from "./grammars/tsrx.tmLanguage.json";
import rippleGrammar from "./grammars/ripple.tmLanguage.json";

/** Cool-biased GitHub themes (D-19 / plan: github-light + github-dark). */
const THEMES = ["github-light", "github-dark"] as const;

export type HighlightTheme = (typeof THEMES)[number];

const GITHUB_CLASS_LANGS = [
  "typescript",
  "tsx",
  "javascript",
  "jsx",
  "json",
  "markdown",
  "html",
  "css",
  "scss",
  "python",
  "rust",
  "go",
  "bash",
  "shell",
  "yaml",
  "toml",
  "sql",
  "dockerfile",
  "diff",
  "plaintext",
] as const;

const tsrxLang = {
  name: "tsrx",
  scopeName: "source.tsrx",
  ...tsrxGrammar,
};

const rippleLang = {
  name: "ripple",
  scopeName: "source.ripple",
  ...rippleGrammar,
};

let highlighterPromise: Promise<Highlighter> | null = null;

/**
 * Singleton Shiki highlighter with GitHub-class langs + in-repo tsrx/ripple grammars (D-19).
 * Custom langs are full TextMate grammars — not TypeScript/JavaScript aliases.
 */
export async function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: [...THEMES],
      langs: [...GITHUB_CLASS_LANGS, tsrxLang, rippleLang],
    });
  }
  return highlighterPromise;
}

/** Map a repo path to a Shiki language id (including .tsrx / .ripple). */
export function languageIdForPath(filePath: string): string {
  const base = filePath.split(/[/\\]/).pop() ?? filePath;
  const lower = base.toLowerCase();
  const dot = lower.lastIndexOf(".");
  const ext = dot >= 0 ? lower.slice(dot) : "";

  switch (ext) {
    case ".tsrx":
      return "tsrx";
    case ".ripple":
      return "ripple";
    case ".ts":
      return "typescript";
    case ".tsx":
      return "tsx";
    case ".js":
    case ".mjs":
    case ".cjs":
      return "javascript";
    case ".jsx":
      return "jsx";
    case ".md":
    case ".markdown":
      return "markdown";
    case ".json":
      return "json";
    case ".html":
    case ".htm":
      return "html";
    case ".css":
      return "css";
    case ".scss":
      return "scss";
    case ".py":
      return "python";
    case ".rs":
      return "rust";
    case ".go":
      return "go";
    case ".sh":
    case ".bash":
      return "bash";
    case ".yml":
    case ".yaml":
      return "yaml";
    case ".toml":
      return "toml";
    case ".sql":
      return "sql";
    case ".dockerfile":
      return "dockerfile";
    case ".diff":
    case ".patch":
      return "diff";
    default:
      if (lower === "dockerfile") return "dockerfile";
      return "plaintext";
  }
}

export async function highlightCode(
  code: string,
  options: { lang: string; theme?: HighlightTheme },
): Promise<string> {
  const highlighter = await getHighlighter();
  const theme = options.theme ?? "github-dark";
  const loaded = highlighter.getLoadedLanguages();
  const lang = loaded.includes(options.lang) ? options.lang : "plaintext";
  const html = highlighter.codeToHtml(code, { lang, theme });
  // Annotate language id for callers/tests — Shiki HTML may omit the lang name.
  return html.replace(
    /<pre(\s)/,
    `<pre data-language="${lang}"$1`,
  );
}
