import { OTPInput, REGEXP_ONLY_DIGITS, type SlotProps } from "input-otp";
import { cn } from "@/lib/utils";

function OtpSlot({ char, isActive, hasFakeCaret }: SlotProps) {
  return (
    <div
      data-slot="input-otp-slot"
      className={cn(
        "relative flex h-11 w-9 shrink-0 items-center justify-center rounded-md border border-input bg-card text-[14px] font-semibold tabular-nums text-foreground transition-colors duration-150",
        isActive && "z-10 ring-2 ring-ring",
      )}
    >
      {char}
      {hasFakeCaret ? (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 flex items-center justify-center"
        >
          <div className="h-5 w-px animate-pulse bg-foreground" />
        </div>
      ) : null}
    </div>
  );
}

export type InputOtpProps = {
  value: string;
  onChange: (value: string) => void;
  onComplete?: (value: string) => void;
  disabled?: boolean;
  id?: string;
  name?: string;
  className?: string;
  "aria-label"?: string;
  "aria-invalid"?: boolean;
  "aria-describedby"?: string;
};

/** Thin Octane/Tailwind wrapper around npm `input-otp` — 8 numeric slots (D-17, D-20). */
export function InputOtp({
  value,
  onChange,
  onComplete,
  disabled,
  id,
  name,
  className,
  ...a11y
}: InputOtpProps) {
  return (
    <OTPInput
      id={id}
      name={name}
      value={value}
      onChange={onChange}
      onComplete={onComplete}
      maxLength={8}
      inputMode="numeric"
      autoComplete="one-time-code"
      pattern={REGEXP_ONLY_DIGITS}
      disabled={disabled}
      containerClassName={cn(
        "group flex w-full items-center justify-between gap-1 has-[:disabled]:opacity-50",
        className,
      )}
      render={({ slots }) => (
        <div className="flex w-full flex-wrap justify-between gap-1">
          {slots.map((slot, idx) => (
            <OtpSlot key={idx} {...slot} />
          ))}
        </div>
      )}
      {...a11y}
    />
  );
}
