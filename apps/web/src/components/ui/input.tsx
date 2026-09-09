import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";

export function Input({ className, ...props }: ComponentProps<"input">) {
  return (
    <input
      data-slot="input"
      className={cn(
        "h-11 w-full rounded-md border border-input bg-card px-3 text-[14px] font-normal text-foreground outline-none transition-colors duration-150 placeholder:text-muted-foreground/90 focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:border-border disabled:bg-muted/40 disabled:text-muted-foreground",
        className,
      )}
      {...props}
    />
  );
}
