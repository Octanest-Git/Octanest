import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const badgeVariants = cva(
  "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[12px] font-semibold leading-none",
  {
    variants: {
      variant: {
        default: "border-border bg-muted text-foreground",
        secondary: "border-secondary/40 bg-secondary/10 text-secondary",
        outline: "border-border bg-transparent text-muted-foreground",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

type BadgeProps = {
  className?: string;
  children?: unknown;
} & VariantProps<typeof badgeVariants>;

export function Badge({ className, variant, children }: BadgeProps) @{
  <span data-slot="badge" className={cn(badgeVariants({ variant }), className)}>
    {children}
  </span>;
}
