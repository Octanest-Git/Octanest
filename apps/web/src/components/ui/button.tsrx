import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-md text-[14px] no-underline transition-[background-color,border-color,color,box-shadow,filter] duration-150 ease-out outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-60",
  {
    variants: {
      variant: {
        // Filled brand CTA (primary action on a page).
        default:
          "bg-primary font-semibold text-primary-foreground shadow-sm hover:brightness-110 disabled:bg-muted disabled:text-muted-foreground disabled:hover:brightness-100 disabled:shadow-none",
        primary:
          "bg-primary font-semibold text-primary-foreground shadow-sm hover:brightness-110 disabled:bg-muted disabled:text-muted-foreground disabled:hover:brightness-100 disabled:shadow-none",
        // Neutral bordered button (GitHub default) — for secondary actions.
        outline:
          "border border-border bg-card font-medium text-foreground shadow-sm hover:bg-accent hover:text-accent-foreground disabled:bg-muted/40 disabled:text-muted-foreground disabled:shadow-none",
        secondary:
          "border-2 border-secondary bg-secondary/5 font-semibold text-secondary hover:bg-secondary/12 disabled:border-secondary/80 disabled:text-secondary/90 disabled:bg-secondary/5",
        ghost:
          "font-medium text-foreground/85 hover:bg-muted hover:text-foreground disabled:text-foreground/70",
        destructive:
          "bg-destructive font-semibold text-destructive-foreground shadow-sm hover:brightness-110 disabled:bg-muted disabled:text-muted-foreground disabled:hover:brightness-100 disabled:shadow-none",
        link: "font-medium text-primary underline-offset-4 hover:underline disabled:text-muted-foreground",
      },
      size: {
        sm: "h-8 px-3 text-[13px]",
        default: "h-9 px-4",
        lg: "h-11 px-6 text-[15px]",
        icon: "h-9 w-9 px-0",
        "icon-sm": "h-8 w-8 px-0",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  },
);

type ButtonProps = {
  className?: string;
  disabled?: boolean;
  type?: "button" | "submit" | "reset";
  title?: string;
  onClick?: () => void;
  children?: unknown;
} & VariantProps<typeof buttonVariants>;

export function Button({ className, variant, size, children, ...rest }: ButtonProps) @{
  <button data-slot="button" className={cn(buttonVariants({ variant, size }), className)} {...rest}>
    {children}
  </button>;
}
