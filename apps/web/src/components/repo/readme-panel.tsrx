import { MarkdownHtml } from "@/components/repo/markdown-body";

type ReadmePanelProps = {
  /** Sanitized HTML from `renderGfm` (SSR or client). */
  html: string;
  fileName: string;
};

/** Sanitized GFM README below the tree (D-18). */
export function ReadmePanel({ html, fileName }: ReadmePanelProps) @{
  <section className="mt-8" aria-labelledby="readme-heading">
    <h2 id="readme-heading" className="mb-3 text-[14px] font-semibold text-foreground">
      {fileName as string}
    </h2>
    <MarkdownHtml
      html={html as string}
      className="rounded-md border border-border bg-card/30 px-6 py-6"
      data-testid="readme-panel"
    />
  </section>;
}
