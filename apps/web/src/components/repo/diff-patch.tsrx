import type { DiffLineKind } from "@/lib/parse-unified-diff";
import { parseUnifiedDiffLines } from "@/lib/parse-unified-diff";
import { cn } from "@/lib/utils";

type DiffPatchProps = {
  path: string;
  status: string;
  patch: string;
};

function lineClass(kind: DiffLineKind): string {
  switch (kind) {
    case "add":
      return "bg-[var(--diff-add-bg)] text-[var(--diff-add-fg)]";
    case "del":
      return "bg-[var(--diff-del-bg)] text-[var(--diff-del-fg)]";
    case "hunk":
      return "bg-[var(--diff-hunk-bg)] text-[var(--diff-hunk-fg)]";
    case "meta":
    case "note":
      return "text-muted-foreground";
    case "ctx":
    default:
      return "text-foreground";
  }
}

/** Unified-diff file block: path header + colorized hunk lines (commit / compare). */
export function DiffPatch({ path, status, patch }: DiffPatchProps) @{
  const lines = parseUnifiedDiffLines(patch).map((line, index) => ({
    ...line,
    index,
  }));

  <section className="overflow-hidden border border-border">
    <h3 className="border-b border-border bg-muted/40 px-3 py-2 font-mono text-[14px] text-foreground">
      {path as string}
      {" "}
      <span className="text-muted-foreground">({status as string})</span>
    </h3>
    @if (lines.length === 0) {
      <p className="px-3 py-3 text-[14px] text-muted-foreground">
        No textual patch for this file.
      </p>
    } @else {
      <div
        className="overflow-x-auto font-mono text-[14px] leading-[1.45]"
        role="table"
        aria-label={`Diff for ${path}`}
      >
        @for (const line of lines; key line.index) {
          <div
            className={cn(
              "whitespace-pre px-3 py-0.5",
              lineClass(line.kind) as string,
            )}
            role="row"
          >
            {line.text as string}
          </div>
        }
      </div>
    }
  </section>
}
