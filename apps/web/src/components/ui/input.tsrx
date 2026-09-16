import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

export function Input({ className, ...props }: ComponentProps<"input">) @{
  <input
    data-slot="input"
    className={cn(
      "h-11 w-full rounded-md border border-input bg-background px-3 text-[14px] font-normal text-foreground outline-none transition-[border-color,box-shadow,background-color] duration-150 placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:cursor-not-allowed disabled:border-border disabled:bg-muted/40 disabled:text-muted-foreground",
      className,
    )}
    {...props}
  />;
}
