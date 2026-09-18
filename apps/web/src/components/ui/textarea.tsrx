import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

export function Textarea({ className, ...props }: ComponentProps<"textarea">) @{
  <textarea
    data-slot="textarea"
    className={cn(
      "min-h-16 w-full rounded-md border border-input bg-background px-3 py-2 text-[14px] font-normal text-foreground outline-none transition-[border-color,box-shadow,background-color] duration-150",
      "placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40",
      "aria-[invalid=true]:border-destructive aria-[invalid=true]:focus-visible:ring-destructive/30",
      "disabled:cursor-not-allowed disabled:border-border disabled:bg-muted/40 disabled:text-muted-foreground",
      className,
    )}
    {...props}
  />;
}
