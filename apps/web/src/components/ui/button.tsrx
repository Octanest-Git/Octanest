import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-md text-[14px] no-underline transition-[background-color,border-color,color,filter] duration-150 ease-out outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed",
  {
    variants: {
      variant: {
        default:
          "bg-primary font-semibold text-primary-foreground hover:brightness-110 disabled:bg-muted disabled:text-muted-foreground disabled:hover:brightness-100",
        secondary:
          "border-2 border-secondary bg-secondary/5 font-semibold text-secondary hover:bg-secondary/12 disabled:border-secondary/80 disabled:text-secondary/90 disabled:bg-secondary/5",
        ghost:
          "font-normal text-foreground/85 hover:bg-muted hover:text-foreground disabled:text-foreground/70",
        destructive:
          "bg-destructive font-semibold text-destructive-foreground hover:brightness-110 disabled:bg-muted disabled:text-muted-foreground disabled:hover:brightness-100",
      },
      size: {
        default: "h-11 px-4",
        icon: "h-11 w-11 px-0",
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
