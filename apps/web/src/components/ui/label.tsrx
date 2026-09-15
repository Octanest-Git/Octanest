import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

export function Label({ className, ...props }: ComponentProps<"label">) @{
  <label
    data-slot="label"
    className={cn(
      "flex items-center gap-2 text-[14px] font-normal leading-[1.4] text-foreground select-none peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
      className,
    )}
    {...props}
  />;
}
