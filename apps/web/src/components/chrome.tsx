import { Link } from "@octanejs/tanstack-router";
import { OctanestMark } from "@/components/octanest-mark";
import { ThemeSelect } from "@/components/theme-select";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function SiteHeader() {
  return (
    <header className="border-b border-border bg-card">
      <div className="mx-auto flex h-16 max-w-6xl items-center gap-4 px-4">
        <Link to="/" className="flex items-center gap-2 no-underline">
          <OctanestMark size={32} />
          <span className="hidden font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground sm:inline">
            Octanest
          </span>
        </Link>
        <div className="ml-auto flex flex-1 items-center justify-end gap-3">
          <Input
            className="hidden max-w-xs sm:block"
            placeholder="Search public code (soon)"
            disabled
            title="Coming soon"
            aria-label="Search public code"
          />
          <ThemeSelect />
          <div role="group" aria-label="Account" className="flex items-center gap-2">
            <Button variant="ghost" disabled title="Coming soon">
              Sign in
            </Button>
            <Button variant="secondary" disabled title="Coming soon">
              Sign up
            </Button>
          </div>
        </div>
      </div>
    </header>
  );
}

export function SiteFooter() {
  return (
    <footer className="border-t border-border py-8 text-[14px] text-muted-foreground">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-4">
        <span>© Octanest</span>
        <Link
          to="/status"
          className="text-foreground underline-offset-4 hover:underline"
        >
          Status
        </Link>
      </div>
    </footer>
  );
}
