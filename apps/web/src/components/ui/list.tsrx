import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

/**
 * Bordered, rounded list container with divided rows.
 * Compose with <ListRow> children. Use an optional <ListHeader> as the first row.
 */
export function List({ className, ...props }: ComponentProps<"div">) @{
  <div
    data-slot="list"
    className={cn("overflow-hidden rounded-lg border border-border bg-card", className)}
    {...props}
  />;
}

/** Muted caption/toolbar row rendered at the top of a <List>. */
export function ListHeader({ className, ...props }: ComponentProps<"div">) @{
  <div
    data-slot="list-header"
    className={cn(
      "flex items-center gap-3 border-b border-border bg-muted/40 px-4 py-2.5 text-[13px] font-semibold text-foreground",
      className,
    )}
    {...props}
  />;
}

/** A single list row. Dividers come from the row's top border (first row flush). */
export function ListRow({ className, ...props }: ComponentProps<"div">) @{
  <div
    data-slot="list-row"
    className={cn(
      "flex items-center gap-3 border-t border-border px-4 py-3 transition-colors first:border-t-0 hover:bg-muted/40",
      className,
    )}
    {...props}
  />;
}
