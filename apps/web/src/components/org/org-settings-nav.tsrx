function navClass(isActive: boolean): string {
  return isActive
    ? "border-b-2 border-primary pb-2 text-[14px] font-semibold text-foreground no-underline"
    : "pb-2 text-[14px] font-normal text-muted-foreground no-underline hover:text-foreground";
}

/** Org settings secondary nav: General | Members | Labels (D-ORG-06 / D-ISS-05). */
export function OrgSettingsNav({
  slug,
  active,
}: {
  slug: string;
  active: "general" | "members" | "labels";
}) @{
  <nav
    className="mb-6 flex flex-wrap gap-4 border-b border-border pb-0"
    aria-label="Organization settings"
  >
    <a
      href={`/${slug}/settings`}
      className={navClass(active === "general") as string}
      aria-current={active === "general" ? "page" : undefined}
    >
      General
    </a>
    <a
      href={`/${slug}/settings/members`}
      className={navClass(active === "members") as string}
      aria-current={active === "members" ? "page" : undefined}
    >
      Members
    </a>
    <a
      href={`/${slug}/settings/labels`}
      className={navClass(active === "labels") as string}
      aria-current={active === "labels" ? "page" : undefined}
    >
      Labels
    </a>
  </nav>;
}
