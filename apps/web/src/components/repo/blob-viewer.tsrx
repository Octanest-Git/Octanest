import { useEffect, useEffectEvent, useState } from "octane";
import type { RepoBlobResponse } from "@octanest/api-client";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { highlightCode, languageIdForPath } from "@/lib/highlight";
import {
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

/** Syntax-highlighted blob with line numbers + #L / #L-L wash (D-19, D-20). */
export function BlobViewer({ owner, repo, blob }: BlobViewerProps) @{
  const [hl, setHl] = useState<string | null>(null);
  const [range, setRange] = useState<{ start: number; end: number } | null>(
    null,
  );
  const rawUrl = rawBlobUrl(owner, repo, blob.ref, blob.path);

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

  const showImage =
    blob.is_binary &&
    isImagePath(blob.path) &&
    !!blob.content &&
    blob.encoding === "base64";

  <div className="rounded-md border border-border bg-card/40">
    @if (blob.truncated) {
      <div
        role="status"
        className="flex flex-wrap items-center gap-3 border-b border-border px-4 py-3"
      >
        <p className="flex-1 text-[14px] text-muted-foreground">
          This file is too large to display. View raw or download.
        </p>
        <a
          href={rawUrl as string}
          className={cn(buttonVariants({ variant: "secondary" }))}
          download
        >
          Download file
        </a>
        <a
          href={rawUrl as string}
          className={cn(buttonVariants({ variant: "ghost" }))}
          target="_blank"
          rel="noreferrer"
        >
          View raw file
        </a>
      </div>
    }

    @if (showImage) {
      <div className="flex justify-center p-6">
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
          <a
            href={rawUrl as string}
            className={cn(
              buttonVariants({ variant: "secondary" }),
              "mt-4 inline-flex",
            )}
            download
          >
            Download file
          </a>
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
              className="sticky left-0 z-10 shrink-0 select-none border-r border-border bg-muted/40 font-mono text-[14px] leading-5 text-muted-foreground"
              aria-label="Line numbers"
            >
              @for (const n of lineNums; key n) {
                const wash =
                  range && n >= range.start && n <= range.end
                    ? "bg-primary/10 text-foreground"
                    : "";
                <button
                  type="button"
                  id={`L${n}` as string}
                  className={
                    (`block min-w-[3rem] px-2 text-right leading-5 hover:text-foreground ${wash}` as string)
                  }
                  onClick={() => goLine(n)}
                  aria-label={`Line ${n}`}
                >
                  {String(n) as string}
                </button>
              }
            </div>
            <div
              className="min-w-0 flex-1 overflow-x-auto font-mono text-[14px] leading-5 [&_pre]:m-0 [&_pre]:bg-transparent! [&_pre]:p-3 [&_code]:font-mono"
              dangerouslySetInnerHTML={{
                __html: (hl ?? `<pre><code>${escapeHtml(text)}</code></pre>`) as string,
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
