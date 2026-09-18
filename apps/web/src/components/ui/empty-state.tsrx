import { cn } from "@/lib/utils";

type EmptyStateProps = {
  /** Leading icon node (e.g. a lucide icon at ~24px). */
  icon?: unknown;
  title: unknown;
  description?: unknown;
  /** Call-to-action node(s) — buttons / links. */
  action?: unknown;
  className?: string;
  /** Compact variant for inline/empty list slots. */
  compact?: boolean;
};

/**
 * Friendly zero/empty state: centered icon, heading, hint, and an optional CTA.
 * The single biggest fix for "bare" pages — use anywhere a list, search, or
 * section can be empty instead of a lone muted sentence.
 */
export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
  compact = false,
}: EmptyStateProps) @{
  <div
    data-slot="empty-state"
    className={cn(
      "flex flex-col items-center justify-center rounded-lg border border-dashed border-border bg-card/40 text-center",
      compact ? "gap-2 px-6 py-8" : "gap-3 px-6 py-14",
      className,
    )}
  >
    @if (icon) {
      <div
        className="flex size-11 items-center justify-center rounded-full bg-muted text-muted-foreground"
        aria-hidden="true"
      >
        {icon}
      </div>;
    }
    <h3 className="font-[family-name:var(--font-display)] text-[16px] font-semibold text-foreground">
      {title}
    </h3>
    @if (description) {
      <p className="max-w-md text-[14px] text-muted-foreground">{description}</p>;
    }
    @if (action) {
      <div className="mt-2 flex flex-wrap items-center justify-center gap-2">{action}</div>;
    }
  </div>;
}
