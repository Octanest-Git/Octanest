import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

/** Bordered surface panel — the base container for grouped content (GitHub "Box"). */
export function Card({ className, ...props }: ComponentProps<"div">) @{
  <div
    data-slot="card"
    className={cn("rounded-lg border border-border bg-card text-card-foreground", className)}
    {...props}
  />;
}

export function CardHeader({ className, ...props }: ComponentProps<"div">) @{
  <div
    data-slot="card-header"
    className={cn("flex flex-col gap-1 border-b border-border px-4 py-3", className)}
    {...props}
  />;
}

export function CardTitle({ className, ...props }: ComponentProps<"h2">) @{
  <h2
    data-slot="card-title"
    className={cn(
      "font-[family-name:var(--font-display)] text-[15px] font-semibold leading-tight text-foreground",
      className,
    )}
    {...props}
  />;
}

export function CardDescription({ className, ...props }: ComponentProps<"p">) @{
  <p
    data-slot="card-description"
    className={cn("text-[13px] text-muted-foreground", className)}
    {...props}
  />;
}

export function CardContent({ className, ...props }: ComponentProps<"div">) @{
  <div data-slot="card-content" className={cn("px-4 py-4", className)} {...props} />;
}

export function CardFooter({ className, ...props }: ComponentProps<"div">) @{
  <div
    data-slot="card-footer"
    className={cn(
      "flex items-center gap-2 border-t border-border bg-muted/30 px-4 py-3",
      className,
    )}
    {...props}
  />;
}
