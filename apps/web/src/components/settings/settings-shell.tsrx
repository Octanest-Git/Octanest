import { cn } from "@/lib/utils";
import {
  AdminSettingsNav,
  type AdminSettingsNavActive,
} from "@/components/admin/admin-settings-nav";
import { SettingsNav, type SettingsNavActive } from "@/components/settings/settings-nav";

/** Shared fixed canvas for account + admin settings (prevents width shift between pages). */
export const SETTINGS_SHELL_MAX = "max-w-5xl";

export type SettingsShellProps = {
  children?: unknown;
  className?: string;
  "data-testid"?: string;
  /** Loading / error / forbidden — same max-width, no sidebar. */
  bare?: boolean;
};

/**
 * Fixed-width settings canvas. Pair with {@link AccountSettingsLayout} /
 * {@link AdminSettingsLayout} for the sidebar grid.
 */
export function SettingsShell(props: SettingsShellProps) @{
  <div
    className={cn("mx-auto w-full px-4 py-10 sm:px-6", SETTINGS_SHELL_MAX, props.className)}
    data-testid={props["data-testid"]}
  >
    {props.children}
  </div>;
}

export type AccountSettingsLayoutProps = {
  active: SettingsNavActive;
  children?: unknown;
  className?: string;
  "data-testid"?: string;
};

/** Account `/settings/*` shell: fixed width + sidebar nav. */
export function AccountSettingsLayout(props: AccountSettingsLayoutProps) @{
  <SettingsShell className={props.className} data-testid={props["data-testid"]}>
    <div className="grid gap-8 md:grid-cols-[14rem_minmax(0,1fr)] md:gap-10">
      <aside className="md:sticky md:top-20 md:self-start">
        <SettingsNav active={props.active} />
      </aside>
      <div className="min-w-0">{props.children}</div>
    </div>
  </SettingsShell>;
}

export type AdminSettingsLayoutProps = {
  active: AdminSettingsNavActive;
  children?: unknown;
  className?: string;
  "data-testid"?: string;
};

/** Instance `/admin/*` shell: fixed width + sidebar nav. */
export function AdminSettingsLayout(props: AdminSettingsLayoutProps) @{
  <SettingsShell className={props.className} data-testid={props["data-testid"]}>
    <div className="grid gap-8 md:grid-cols-[14rem_minmax(0,1fr)] md:gap-10">
      <aside className="md:sticky md:top-20 md:self-start">
        <AdminSettingsNav active={props.active} />
      </aside>
      <div className="min-w-0">{props.children}</div>
    </div>
  </SettingsShell>;
}
