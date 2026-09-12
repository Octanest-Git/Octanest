import { useEffect, useState } from "octane";
import { renderGfm } from "@/lib/markdown";
import { Skeleton } from "@/components/ui/skeleton";

type ReadmePanelProps = {
  markdown: string;
  fileName: string;
};

/** Sanitized GFM README below the tree (D-18). */
export function ReadmePanel({ markdown, fileName }: ReadmePanelProps) @{
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const out = await renderGfm(markdown);
      if (!cancelled) setHtml(out);
    })();
    return () => {
      cancelled = true;
    };
  }, [markdown]);

  <section className="mt-8" aria-labelledby="readme-heading">
    <h2
      id="readme-heading"
      className="mb-3 text-[14px] font-semibold text-foreground"
    >
      {fileName as string}
    </h2>
    @if (html === null) {
      <div className="flex flex-col gap-2" aria-busy="true">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-5/6" />
      </div>
    } @else {
      <div
        className="prose-octanest max-w-none rounded-md border border-border bg-card/30 px-6 py-6 text-[16px] leading-[1.5] text-foreground [&_a]:text-primary [&_code]:rounded [&_code]:bg-muted/50 [&_code]:px-1 [&_h1]:mb-4 [&_h1]:font-[family-name:var(--font-display)] [&_h1]:text-[24px] [&_h1]:font-semibold [&_h2]:mb-3 [&_h2]:text-[16px] [&_h2]:font-semibold [&_pre]:overflow-x-auto [&_pre]:rounded-md [&_pre]:bg-muted/40 [&_pre]:p-3 [&_table]:w-full [&_td]:border [&_td]:border-border [&_td]:px-2 [&_th]:border [&_th]:border-border [&_th]:px-2"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    }
  </section>
}
