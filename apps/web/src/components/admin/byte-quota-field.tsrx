import { Check, ChevronDown } from "@octanejs/lucide";
import {
  SelectIcon,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectList,
  SelectPopup,
  SelectPortal,
  SelectPositioner,
  SelectRoot,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { BYTE_UNIT_OPTIONS, type ByteUnit } from "@/lib/byte-units";

type ByteQuotaFieldProps = {
  id: string;
  label: string;
  value: string;
  unit: ByteUnit;
  disabled?: boolean;
  overridden?: boolean;
  onValueChange: (next: string) => void;
  onUnitChange: (next: ByteUnit) => void;
  onBlur?: () => void;
};

/** Numeric amount + binary unit select for admin quota overrides. */
export function ByteQuotaField({
  id,
  label,
  value,
  unit,
  disabled = false,
  overridden = false,
  onValueChange,
  onUnitChange,
  onBlur,
}: ByteQuotaFieldProps) @{
  const unitLabel = BYTE_UNIT_OPTIONS.find((o) => o.value === unit)?.label ?? unit;
  const unitSelectId = `${id}-unit`;

  <div className="flex flex-col gap-2">
    <Label htmlFor={id}>{label as string}</Label>
    <div className="flex flex-wrap items-stretch gap-2">
      <Input
        id={id}
        className="min-w-[8rem] flex-1"
        inputMode="decimal"
        value={value}
        onBlur={onBlur}
        onInput={(e) => onValueChange((e.currentTarget as HTMLInputElement).value)}
        disabled={disabled}
        data-testid={`${id}-amount`}
      />
      <SelectRoot
        value={unit}
        onValueChange={(next) => {
          if (typeof next === "string") onUnitChange(next as ByteUnit);
        }}
      >
        <SelectTrigger
          id={unitSelectId}
          className="w-[6.5rem] shrink-0"
          aria-label={`${label} unit`}
          disabled={disabled}
          data-testid={`${id}-unit`}
        >
          <SelectValue>{unitLabel as string}</SelectValue>
          <SelectIcon>
            <ChevronDown aria-hidden size={16} />
          </SelectIcon>
        </SelectTrigger>
        <SelectPortal>
          <SelectPositioner sideOffset={4}>
            <SelectPopup>
              <SelectList>
                @for (const opt of BYTE_UNIT_OPTIONS; key opt.value) {
                  <SelectItem value={opt.value}>
                    <SelectItemIndicator>
                      <Check aria-hidden size={16} className="text-primary" />
                    </SelectItemIndicator>
                    <SelectItemText>{opt.label as string}</SelectItemText>
                  </SelectItem>;
                }
              </SelectList>
            </SelectPopup>
          </SelectPositioner>
        </SelectPortal>
      </SelectRoot>
    </div>
    @if (overridden) {
      <span className="text-[13px] text-muted-foreground">Override active</span>;
    }
  </div>;
}
