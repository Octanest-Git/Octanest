import { cn } from "@/lib/utils";

type Variant = "primary" | "secondary" | "ghost";

export function Button(
  props: {
    className?: string;
    variant?: Variant;
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    title?: string;
    onClick?: () => void;
    children?: unknown;
  },
) {
  const { className, variant = "primary", children, ...rest } = props;
  return (
    <button
      className={cn(
        "inline-flex h-11 items-center justify-center rounded-md px-4 text-[14px] font-semibold transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-accent-cool)] disabled:cursor-not-allowed disabled:opacity-50",
        variant === "primary" &&
          "bg-[var(--color-accent-cool)] text-white hover:brightness-110",
        variant === "secondary" &&
          "border border-[var(--color-accent-warm)] text-[var(--color-accent-warm)] hover:bg-[color-mix(in_srgb,var(--color-accent-warm)_12%,transparent)]",
        variant === "ghost" &&
          "text-[var(--color-muted)] hover:text-[var(--color-text)]",
        className,
      )}
      {...rest}
    >
      {children}
    </button>
  );
}
