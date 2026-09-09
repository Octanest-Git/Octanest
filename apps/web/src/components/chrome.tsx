import { Link } from "@octanejs/tanstack-router";
import { Button } from "@/components/ui/button";
import { applyTheme, readThemePreference, type ThemePreference } from "@/lib/theme";
import { useEffect, useState } from "octane";

export function SiteHeader() {
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

  function onThemeChange(next: ThemePreference) {
    setTheme(next);
    applyTheme(next);
  }

  return (
    <header className="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="mx-auto flex h-16 max-w-6xl items-center gap-4 px-4">
        <Link to="/" className="flex items-center gap-2 font-[family-name:var(--font-display)] text-[24px] font-semibold text-[var(--color-text)] no-underline">
          <img src="/octanest-mark.png" alt="" width={32} height={32} />
          Octanest
        </Link>
        <div className="ml-auto flex flex-1 items-center justify-end gap-3">
          <input
            className="hidden h-11 w-full max-w-xs rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-[14px] text-[var(--color-muted)] sm:block"
            placeholder="Search public code (soon)"
            disabled
            title="Coming soon"
          />
          <select
            aria-label="Theme"
            className="h-11 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-[14px]"
            value={theme}
            onChange={(e) => onThemeChange(e.target.value as ThemePreference)}
          >
            <option value="system">System</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
          <Button variant="ghost" disabled title="Coming soon">
            Sign in
          </Button>
          <Button variant="secondary" disabled title="Coming soon">
            Sign up
          </Button>
        </div>
      </div>
    </header>
  );
}

export function SiteFooter() {
  return (
    <footer className="border-t border-[var(--color-border)] py-8 text-[14px] text-[var(--color-muted)]">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-4">
        <span>© Octanest</span>
        <Link to="/status" className="text-[var(--color-text)] underline-offset-4 hover:underline">
          Status
        </Link>
      </div>
    </footer>
  );
}
