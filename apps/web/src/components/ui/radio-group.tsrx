import { Radio as RadioPrimitive } from "@octanejs/base-ui/radio";
import { RadioGroup as RadioGroupPrimitive } from "@octanejs/base-ui/radio-group";
import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

export function RadioGroup({
  className,
  ...props
}: ComponentProps<typeof RadioGroupPrimitive>) @{
  <RadioGroupPrimitive
    data-slot="radio-group"
    className={cn("flex flex-col gap-3", className)}
    {...props}
  />
}

export function RadioGroupItem({
  className,
  ...props
}: ComponentProps<typeof RadioPrimitive.Root>) @{
  <RadioPrimitive.Root
    data-slot="radio-group-item"
    className={cn(
      "peer relative inline-flex size-5 shrink-0 items-center justify-center rounded-full border border-input bg-card outline-none transition-colors",
      "after:absolute after:-inset-y-2 after:-inset-x-3",
      "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      "data-[checked]:border-primary",
      "disabled:cursor-not-allowed disabled:opacity-50",
      className,
    )}
    {...props}
  >
    <RadioPrimitive.Indicator
      data-slot="radio-group-indicator"
      className="flex items-center justify-center"
    >
      <span className="size-2.5 rounded-full bg-primary" />
    </RadioPrimitive.Indicator>
  </RadioPrimitive.Root>
}
