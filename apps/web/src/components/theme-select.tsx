import { Check, ChevronDown, Monitor, Moon, Sun } from "lucide-react";
import { useEffect, useState } from "octane";
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
import { applyTheme, readThemePreference, type ThemePreference } from "@/lib/theme";

const OPTIONS: { value: ThemePreference; label: string; Icon: typeof Monitor }[] = [
  { value: "system", label: "System", Icon: Monitor },
  { value: "light", label: "Light", Icon: Sun },
  { value: "dark", label: "Dark", Icon: Moon },
];

export function ThemeSelect() {
  const [theme, setTheme] = useState<ThemePreference>("system");

  useEffect(() => {
    const pref = readThemePreference();
    setTheme(pref);
    applyTheme(pref);
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => {
      if (readThemePreference() === "system") applyTheme("system");
    };
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  function onValueChange(next: ThemePreference) {
    setTheme(next);
    applyTheme(next);
  }

  const current = OPTIONS.find((o) => o.value === theme) ?? OPTIONS[0];

  return (
    <SelectRoot value={theme} onValueChange={onValueChange}>
      <SelectTrigger aria-label="Theme" className="min-w-[7.5rem]">
        <span className="flex items-center gap-2">
          <current.Icon aria-hidden size={16} />
          <SelectValue>{current.label}</SelectValue>
        </span>
        <SelectIcon>
          <ChevronDown aria-hidden size={16} />
        </SelectIcon>
      </SelectTrigger>
      <SelectPortal>
        <SelectPositioner sideOffset={4}>
          <SelectPopup>
            <SelectList>
              {OPTIONS.map(({ value, label, Icon }) => (
                <SelectItem key={value} value={value}>
                  <SelectItemIndicator>
                    <Check aria-hidden size={16} className="text-primary" />
                  </SelectItemIndicator>
                  <Icon aria-hidden size={16} />
                  <SelectItemText>{label}</SelectItemText>
                </SelectItem>
              ))}
            </SelectList>
          </SelectPopup>
        </SelectPositioner>
      </SelectPortal>
    </SelectRoot>
  );
}
