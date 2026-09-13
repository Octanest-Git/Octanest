import { Check, Copy, FileCode2 } from "@octanejs/lucide";
import { useEffect, useEffectEvent, useState } from "octane";
import type { RepoBlobResponse } from "@octanest/api-client";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { highlightCode, languageIdForPath } from "@/lib/highlight";
import {
  blameHref,
  formatFileSize,
  imageMimeForPath,
  isImagePath,
  parseLineHash,
  rawBlobUrl,
} from "@/lib/repo-browse";
import { resolveTheme, readThemePreference } from "@/lib/theme";
import { cn } from "@/lib/utils";

type BlobViewerProps = {
  owner: string;
  repo: string;
  blob: RepoBlobResponse;
};

function fileBaseName(path: string): string {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] || path || "file";
}

function lineCountLabel(n: number): string {
  return n === 1 ? "1 line" : `${n} lines`;
}

/** Syntax-highlighted blob with forge-style header, line numbers, #L permalinks (D-19, D-20). */
export function BlobViewer({ owner, repo, blob }: BlobViewerProps) @{
  const [hl, setHl] = useState<string | null>(null);
  const [range, setRange] = useState<{ start: number; end: number } | null>(
    null,
  );
  const [copied, setCopied] = useState(false);
  const rawUrl = rawBlobUrl(owner, repo, blob.ref, blob.path);
  const blameUrl = blameHref(owner, repo, blob.ref, blob.path);
  const baseName = fileBaseName(blob.path);

  const text =
    !blob.is_binary && blob.encoding === "utf-8" && blob.content
      ? blob.content
      : "";
  const rawLines = text.length > 0 ? text.split("\n") : [];
  const lineCount =
    rawLines.length > 0 && rawLines[rawLines.length - 1] === ""
      ? rawLines.length - 1
      : rawLines.length;
  const lineNums = Array.from({ length: lineCount }, (_, i) => i + 1);

  const syncHash = useEffectEvent(() => {
    if (typeof window === "undefined") return;
    setRange(parseLineHash(window.location.hash));
  });

  useEffect(() => {
    syncHash();
    window.addEventListener("hashchange", syncHash);
    return () => window.removeEventListener("hashchange", syncHash);
  }, []);

  useEffect(() => {
    if (
      blob.is_binary ||
      blob.truncated ||
      blob.encoding !== "utf-8" ||
      !blob.content
    ) {
      setHl(null);
      return;
    }
    let cancelled = false;
    (async () => {
      const theme =
        resolveTheme(readThemePreference()) === "dark"
          ? "github-dark"
          : "github-light";
      const out = await highlightCode(blob.content ?? "", {
        lang: languageIdForPath(blob.path),
        theme,
      });
      if (!cancelled) setHl(out);
    })();
    return () => {
      cancelled = true;
    };
  }, [blob.content, blob.path, blob.is_binary, blob.truncated, blob.encoding]);

  useEffect(() => {
    if (!range || typeof document === "undefined") return;
    const el = document.getElementById(`L${range.start}`);
    el?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [range, hl]);

  function goLine(n: number) {
    window.history.replaceState(null, "", `#L${n}`);
    setRange({ start: n, end: n });
  }

  async function copyContents() {
    if (!text || typeof navigator === "undefined" || !navigator.clipboard) {
      return;
    }
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      /* ignore */
    }
  }

  const showImage =
    blob.is_binary &&
    isImagePath(blob.path) &&
    !!blob.content &&
    blob.encoding === "base64";

  const metaBits: string[] = [];
  if (!blob.is_binary && lineCount > 0) {
    metaBits.push(lineCountLabel(lineCount));
  }
  metaBits.push(formatFileSize(blob.size));

  <div className="overflow-hidden rounded-md border border-border bg-card">
    <div className="flex flex-wrap items-center gap-3 border-b border-border bg-muted/35 px-3 py-2">
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <FileCode2
          aria-hidden
          size={16}
          className="shrink-0 text-muted-foreground"
        />
        <span
          className="truncate font-mono text-[14px] font-semibold text-foreground"
          title={blob.path as string}
        >
          {baseName as string}
        </span>
        <span className="shrink-0 text-[13px] text-muted-foreground">
          {metaBits.join(" · ") as string}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-1">
        <a
          href={rawUrl as string}
          className={cn(
            buttonVariants({ variant: "ghost" }),
            "h-8 px-2.5 text-[13px]",
          )}
          target="_blank"
          rel="noreferrer"
        >
          Raw
        </a>
        <a
          href={blameUrl as string}
          className={cn(
            buttonVariants({ variant: "ghost" }),
            "h-8 px-2.5 text-[13px]",
          )}
        >
          Blame
        </a>
        @if (!blob.is_binary && text) {
          <button
            type="button"
            className={cn(
              buttonVariants({ variant: "ghost" }),
              "h-8 gap-1.5 px-2.5 text-[13px]",
            )}
            onClick={() => void copyContents()}
            aria-label={copied ? "Copied" : "Copy file contents"}
            title={copied ? "Copied" : "Copy file contents"}
          >
            @if (copied) {
              <Check aria-hidden size={14} />
            } @else {
              <Copy aria-hidden size={14} />
            }
            {(copied ? "Copied" : "Copy") as string}
          </button>
        }
        @if (blob.truncated || blob.is_binary) {
          <a
            href={rawUrl as string}
            className={cn(
              buttonVariants({ variant: "secondary" }),
              "h-8 px-2.5 text-[13px]",
            )}
            download
          >
            Download
          </a>
        }
      </div>
    </div>

    @if (blob.truncated) {
      <div role="status" className="border-b border-border px-4 py-3">
        <p className="text-[14px] text-muted-foreground">
          This file is too large to display. View raw or download.
        </p>
      </div>
    }

    @if (showImage) {
      <div className="flex justify-center bg-muted/20 p-6">
        <img
          src={
            (`data:${imageMimeForPath(blob.path)};base64,${blob.content}` as string)
          }
          alt={blob.path as string}
          className="max-h-[70vh] max-w-full"
        />
      </div>
    } @else {
      @if (blob.is_binary || blob.encoding !== "utf-8") {
        <div className="px-6 py-10 text-center">
          <p className="text-[16px] text-muted-foreground">
            This file can't be previewed. Download it instead.
          </p>
        </div>
      } @else {
        @if (hl === null && !blob.truncated) {
          <div className="flex flex-col gap-2 p-4" aria-busy="true">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-5/6" />
            <Skeleton className="h-4 w-4/5" />
          </div>
        } @else {
          <div className="flex overflow-x-auto">
            <div
              className="sticky left-0 z-10 shrink-0 select-none border-r border-border bg-muted/50 py-3 font-mono text-[12px] leading-5 text-muted-foreground"
              aria-label="Line numbers"
            >
              @for (const n of lineNums; key n) {
                const wash =
                  range && n >= range.start && n <= range.end
                    ? "bg-primary/15 text-foreground"
                    : "";
                <button
                  type="button"
                  id={`L${n}` as string}
                  className={
                    (`block h-5 min-w-[3.25rem] px-2 text-right leading-5 hover:text-foreground ${wash}` as string)
                  }
                  onClick={() => goLine(n)}
                  aria-label={`Line ${n}`}
                >
                  {String(n) as string}
                </button>
              }
            </div>
            <div
              className="blob-code min-w-0 flex-1 overflow-x-auto py-3 font-mono text-[12px] leading-5 [&_pre]:m-0 [&_pre]:bg-transparent! [&_pre]:p-0 [&_pre]:leading-5 [&_code]:block [&_code]:font-mono [&_code]:leading-5 [&_code]:px-3"
              dangerouslySetInnerHTML={{
                __html: (hl ??
                  `<pre><code>${escapeHtml(text)}</code></pre>`) as string,
              }}
            />
          </div>
        }
      }
    }
  </div>
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
