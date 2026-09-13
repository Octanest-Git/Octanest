type SettingsNavProps = {
  active: "profile" | "tokens";
};

function navClass(isActive: boolean): string {
  return isActive
    ? "border-b-2 border-primary pb-2 text-[14px] font-semibold text-foreground no-underline"
    : "pb-2 text-[14px] font-normal text-muted-foreground no-underline hover:text-foreground";
}

/** Settings secondary link-row: Profile | Personal access tokens (D-14). */
export function SettingsNav({ active }: SettingsNavProps) @{
  <nav
    className="mb-6 flex flex-wrap gap-4 border-b border-border pb-0"
    aria-label="Settings"
  >
    <a
      href="/settings/profile"
      className={navClass(active === "profile") as string}
      aria-current={active === "profile" ? "page" : undefined}
    >
      Profile
    </a>
    <a
      href="/settings/tokens"
      className={navClass(active === "tokens") as string}
      aria-current={active === "tokens" ? "page" : undefined}
    >
      Personal access tokens
    </a>
  </nav>
}
