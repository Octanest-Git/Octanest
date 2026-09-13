/** Line kinds in a unified diff patch (commit / compare file hunks). */
export type DiffLineKind =
  | "meta"
  | "hunk"
  | "add"
  | "del"
  | "ctx"
  | "note";

export type DiffLine = {
  kind: DiffLineKind;
  text: string;
};

/**
 * Classify unified-diff lines for styled rendering.
 * Keeps raw text (including leading `+`/`-`/` `) so copy-paste stays faithful.
 */
export function parseUnifiedDiffLines(patch: string): DiffLine[] {
  if (!patch) return [];
  const raw = patch.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const parts = raw.split("\n");
  // Drop a single trailing empty segment from a final newline.
  if (parts.length > 0 && parts[parts.length - 1] === "") {
    parts.pop();
  }
  return parts.map((text) => ({ kind: classifyDiffLine(text), text }));
}

export function classifyDiffLine(line: string): DiffLineKind {
  if (line.startsWith("@@")) return "hunk";
  if (line === "\\ No newline at end of file") return "note";
  if (
    line.startsWith("diff --git ") ||
    line.startsWith("index ") ||
    line.startsWith("new file mode ") ||
    line.startsWith("deleted file mode ") ||
    line.startsWith("old mode ") ||
    line.startsWith("new mode ") ||
    line.startsWith("similarity index ") ||
    line.startsWith("rename from ") ||
    line.startsWith("rename to ") ||
    line.startsWith("copy from ") ||
    line.startsWith("copy to ") ||
    line.startsWith("Binary files ") ||
    line.startsWith("--- ") ||
    line.startsWith("+++ ")
  ) {
    return "meta";
  }
  if (line.startsWith("+")) return "add";
  if (line.startsWith("-")) return "del";
  if (line.startsWith(" ") || line === "") return "ctx";
  return "meta";
}
