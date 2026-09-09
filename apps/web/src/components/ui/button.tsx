import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 rounded-md text-[14px] no-underline transition-[background-color,border-color,color,filter] duration-150 ease-out outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary font-semibold text-primary-foreground hover:brightness-110",
        secondary:
          "border border-secondary font-semibold text-secondary hover:bg-secondary/10",
        ghost: "font-normal text-muted-foreground hover:text-foreground",
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

export function Button({ className, variant, size, children, ...rest }: ButtonProps) {
  return (
    <button
      data-slot="button"
      className={cn(buttonVariants({ variant, size }), className)}
      {...rest}
    >
      {children}
    </button>
  );
}
