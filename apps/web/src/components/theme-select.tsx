import { Check, Monitor, Moon, Sun } from "@octanejs/lucide";
import { useEffect, useEffectEvent, useState } from "octane";
import { applyTheme, readThemePreference, type ThemePreference } from "@/lib/theme";
import { cn } from "@/lib/utils";

const OPTIONS: { value: ThemePreference; label: string; Icon: typeof Monitor }[] = [
  { value: "system", label: "System", Icon: Monitor },
  { value: "light", label: "Light", Icon: Sun },
  { value: "dark", label: "Dark", Icon: Moon },
];

type ThemeSelectProps = {
  /** compact = header icon + menu; panel = inline list for mobile nav */
  layout?: "compact" | "panel";
  className?: string;
  onChosen?: () => void;
};

/**
 * Theme control: compact header trigger, or inline panel for mobile menus.
 * Avoids Base UI Select (body scroll-lock).
 */
export function ThemeSelect({
  layout = "compact",
  className,
  onChosen,
}: ThemeSelectProps) {
  const [theme, setTheme] = useState<ThemePreference>("system");
  const [open, setOpen] = useState(false);

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

  const onDocPointer = useEffectEvent((event: PointerEvent) => {
    const t = event.target;
    if (!(t instanceof Element)) return;
    if (t.closest("[data-theme-menu]")) return;
    setOpen(false);
  });

  const onDocKey = useEffectEvent((event: KeyboardEvent) => {
    if (event.key === "Escape") setOpen(false);
  });

  useEffect(() => {
    if (!open || layout !== "compact") return;
    document.addEventListener("pointerdown", onDocPointer);
    document.addEventListener("keydown", onDocKey);
    return () => {
      document.removeEventListener("pointerdown", onDocPointer);
      document.removeEventListener("keydown", onDocKey);
    };
  }, [open, layout]);

  function choose(next: ThemePreference) {
    setTheme(next);
    applyTheme(next);
    setOpen(false);
    onChosen?.();
  }

  const current = OPTIONS.find((o) => o.value === theme) ?? OPTIONS[0];

  if (layout === "panel") {
    return (
      <div
        role="listbox"
        aria-label="Theme"
        className={cn("flex flex-col gap-1", className)}
      >
        <p className="px-1 text-[12px] font-medium uppercase tracking-wide text-muted-foreground">
          Theme
        </p>
        <div className="grid grid-cols-3 gap-1">
          {OPTIONS.map(({ value, label, Icon }) => {
            const selected = theme === value;
            return (
              <button
                key={value}
                type="button"
                role="option"
                aria-selected={selected}
                className={cn(
                  "inline-flex h-11 flex-col items-center justify-center gap-0.5 rounded-md border border-border bg-card text-[12px] font-medium",
                  "outline-none transition-colors hover:border-foreground/30 hover:bg-muted",
                  "focus-visible:ring-2 focus-visible:ring-ring",
                  selected && "border-primary bg-muted text-primary",
                )}
                onClick={() => choose(value)}
              >
                <Icon aria-hidden size={16} />
                {label}
              </button>
            );
          })}
        </div>
      </div>
    );
  }

  return (
    <div className={cn("relative", className)} data-theme-menu>
      <button
        type="button"
        className={cn(
          "inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground",
          "transition-colors duration-150 hover:border-foreground/30 hover:bg-muted",
          "outline-none focus-visible:ring-2 focus-visible:ring-ring",
          open && "border-primary bg-muted",
        )}
        aria-label={`Theme: ${current.label}`}
        aria-haspopup="listbox"
        aria-expanded={open}
        title={`Theme: ${current.label}`}
        onClick={() => setOpen((v) => !v)}
      >
        <current.Icon aria-hidden size={18} />
      </button>

      {open ? (
        <div
          role="listbox"
          aria-label="Theme"
          className="absolute right-0 top-[calc(100%+0.35rem)] z-50 min-w-[10.5rem] rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-lg"
        >
          {OPTIONS.map(({ value, label, Icon }) => {
            const selected = theme === value;
            return (
              <button
                key={value}
                type="button"
                role="option"
                aria-selected={selected}
                className={cn(
                  "flex h-10 w-full items-center gap-2 rounded-sm px-2 text-left text-[14px] outline-none",
                  "hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent",
                  selected && "text-primary",
                )}
                onClick={() => choose(value)}
              >
                <Icon aria-hidden size={16} />
                <span className="flex-1 font-medium">{label}</span>
                {selected ? <Check aria-hidden size={16} className="text-primary" /> : null}
              </button>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
