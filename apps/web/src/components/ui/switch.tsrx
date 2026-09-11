import { Switch as SwitchPrimitive } from "@octanejs/base-ui/switch";
import type { ComponentProps } from "octane";
import { cn } from "@/lib/utils";

export function Switch({
  className,
  ...props
}: ComponentProps<typeof SwitchPrimitive.Root>) @{
  <SwitchPrimitive.Root
    data-slot="switch"
    className={cn(
      "peer relative inline-flex h-6 w-11 shrink-0 items-center rounded-full border border-transparent bg-input outline-none transition-colors",
      "after:absolute after:-inset-y-2.5 after:-inset-x-1",
      "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      "data-[checked]:bg-primary",
      "disabled:cursor-not-allowed disabled:opacity-50",
      className,
    )}
    {...props}
  >
    <SwitchPrimitive.Thumb
      data-slot="switch-thumb"
      className={cn(
        "pointer-events-none block size-5 rounded-full bg-card shadow-sm transition-transform",
        "translate-x-0.5 data-[checked]:translate-x-[1.375rem]",
      )}
    />
  </SwitchPrimitive.Root>
}
