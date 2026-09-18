import { cn } from "@/lib/utils";

type PageHeaderProps = {
  title: unknown;
  description?: unknown;
  /** Leading icon node (e.g. a lucide icon). */
  icon?: unknown;
  /** Right-aligned actions (buttons, menus). */
  actions?: unknown;
  className?: string;
  /** Renders a bottom divider (GitHub subheader rule). Default true. */
  divider?: boolean;
};

/**
 * Consistent page-top header: optional icon, title, description, and right-side
 * actions, with an optional bottom rule. Replaces ad-hoc <h1> blocks so every
 * page opens the same way.
 */
export function PageHeader({
  title,
  description,
  icon,
  actions,
  className,
  divider = true,
}: PageHeaderProps) @{
  <div
    data-slot="page-header"
    className={cn(
      "flex flex-col gap-3 pb-4 sm:flex-row sm:items-start sm:justify-between",
      divider && "border-b border-border",
      className,
    )}
  >
    <div className="flex min-w-0 items-start gap-3">
      @if (icon) {
        <span className="mt-0.5 shrink-0 text-muted-foreground" aria-hidden="true">
          {icon}
        </span>;
      }
      <div className="min-w-0">
        <h1 className="font-[family-name:var(--font-display)] text-[22px] font-semibold leading-tight text-foreground">
          {title}
        </h1>
        @if (description) {
          <p className="mt-1 max-w-2xl text-[14px] text-muted-foreground">{description}</p>;
        }
      </div>
    </div>
    @if (actions) {
      <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div>;
    }
  </div>;
}
