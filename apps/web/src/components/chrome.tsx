import { Menu, X } from "@octanejs/lucide";
import { Link, useNavigate } from "@octanejs/tanstack-router";
import type { UserPublic } from "@octanest/api-client";
import { useEffect, useEffectEvent, useState } from "octane";
import { GlobalSearch } from "@/components/global-search";
import { OctanestMark } from "@/components/octanest-mark";
import { ThemeSelect } from "@/components/theme-select";
import { buttonVariants } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { apiClient } from "@/lib/api-client";
import { cn } from "@/lib/utils";

function initialsFor(user: UserPublic): string {
  const source = user.display_name?.trim() || user.username || "?";
  const parts = source.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return `${parts[0]![0] ?? ""}${parts[1]![0] ?? ""}`.toUpperCase();
  }
  return source.slice(0, 2).toUpperCase();
}

function AccountAvatar({ user }: { user: UserPublic }) {
  if (user.avatar_url) {
    return (
      <img
        src={user.avatar_url}
        alt=""
        width={28}
        height={28}
        className="octanest-squircle size-7 shrink-0 object-cover"
      />
    );
  }
  return (
    <span
      aria-hidden
      className="octanest-squircle inline-flex size-7 shrink-0 items-center justify-center bg-muted text-[11px] font-semibold text-foreground"
    >
      {initialsFor(user)}
    </span>
  );
}

function AccountMenu({
  user,
  stacked = false,
  onNavigated,
}: {
  user: UserPublic;
  stacked?: boolean;
  onNavigated?: () => void;
}) {
  const navigate = useNavigate();

  async function logout() {
    try {
      await apiClient.auth.logout();
    } catch {
      /* still clear local chrome */
    }
    onNavigated?.();
    window.location.assign("/");
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className={cn(
          "inline-flex h-11 items-center gap-2 rounded-md px-2 text-[14px] font-normal text-foreground outline-none",
          "hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
          stacked && "w-full justify-center",
        )}
        aria-label="Account menu"
      >
        <AccountAvatar user={user} />
        <span className={cn("text-muted-foreground", stacked ? "inline" : "hidden md:inline")}>
          @{user.username}
        </span>
      </DropdownMenuTrigger>
      <DropdownMenuContent align={stacked ? "center" : "end"} className="min-w-48">
        <DropdownMenuItem
          onClick={() => {
            onNavigated?.();
            window.location.assign("/settings/profile");
          }}
        >
          Profile
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => {
            onNavigated?.();
            void navigate({ to: "/dashboard" });
          }}
        >
          Dashboard
        </DropdownMenuItem>
        {user.is_admin ? (
          <DropdownMenuItem
            onClick={() => {
              onNavigated?.();
              window.location.assign("/admin/auth");
            }}
          >
            Auth settings
          </DropdownMenuItem>
        ) : null}
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => void logout()}>Log out</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function AccountActions({
  className,
  stacked = false,
  onNavigated,
}: {
  className?: string;
  stacked?: boolean;
  onNavigated?: () => void;
}) {
  const [user, setUser] = useState<UserPublic | null | undefined>(undefined);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await apiClient.auth.me();
        if (cancelled) return;
        setUser(res.ok ? res.data : null);
      } catch {
        if (cancelled) return;
        setUser(null);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (user === undefined) {
    return (
      <div
        role="group"
        aria-label="Account"
        className={cn(
          stacked ? "flex flex-col gap-2" : "flex items-center gap-2",
          className,
        )}
      >
        <span className="inline-flex h-11 min-w-20 items-center justify-center text-[14px] text-muted-foreground">
          …
        </span>
      </div>
    );
  }

  if (user) {
    return (
      <div
        role="group"
        aria-label="Account"
        className={cn(
          stacked ? "flex flex-col gap-2" : "flex items-center gap-2",
          className,
        )}
      >
        <AccountMenu user={user} stacked={stacked} onNavigated={onNavigated} />
      </div>
    );
  }

  return (
    <div
      role="group"
      aria-label="Account"
      className={cn(
        stacked ? "flex flex-col gap-2" : "flex items-center gap-2",
        className,
      )}
    >
      <Link
        to="/login"
        preload="intent"
        className={cn(
          buttonVariants({ variant: "ghost" }),
          stacked ? "h-11 w-full justify-center" : "h-11 px-3",
        )}
        onClick={onNavigated}
      >
        Sign in
      </Link>
      <Link
        to="/signup"
        preload="intent"
        className={cn(
          buttonVariants({ variant: "secondary" }),
          stacked ? "h-11 w-full justify-center" : "h-11 px-3",
        )}
        onClick={onNavigated}
      >
        Sign up
      </Link>
    </div>
  );
}

export function SiteHeader() {
  const [menuOpen, setMenuOpen] = useState(false);

  const closeMenu = useEffectEvent(() => setMenuOpen(false));

  const onDocPointer = useEffectEvent((event: PointerEvent) => {
    const t = event.target;
    if (!(t instanceof Element)) return;
    if (t.closest("[data-site-header]")) return;
    setMenuOpen(false);
  });

  const onDocKey = useEffectEvent((event: KeyboardEvent) => {
    if (event.key === "Escape") setMenuOpen(false);
  });

  useEffect(() => {
    if (!menuOpen) return;
    document.addEventListener("pointerdown", onDocPointer);
    document.addEventListener("keydown", onDocKey);
    return () => {
      document.removeEventListener("pointerdown", onDocPointer);
      document.removeEventListener("keydown", onDocKey);
    };
  }, [menuOpen]);

  useEffect(() => {
    const mq = window.matchMedia("(min-width: 768px)");
    const onChange = () => {
      if (mq.matches) closeMenu();
    };
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  return (
    <header
      data-site-header
      className="sticky top-0 z-40 border-b border-border bg-card"
    >
      <div className="mx-auto grid h-14 max-w-6xl grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-2 px-3 sm:gap-x-3 sm:px-4 md:h-16 md:gap-x-4">
        <Link
          to="/"
          preload="intent"
          className="col-start-1 row-start-1 flex shrink-0 items-center gap-2 no-underline"
          onClick={closeMenu}
        >
          <OctanestMark size={32} />
          <span className="hidden font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground md:inline">
            Octanest
          </span>
        </Link>

        <GlobalSearch className="col-start-2 row-start-1" />

        <div className="col-start-3 row-start-1 hidden items-center justify-end gap-2 md:flex">
          <ThemeSelect />
          <AccountActions />
        </div>

        <button
          type="button"
          className="col-start-3 row-start-1 inline-flex h-11 w-11 shrink-0 items-center justify-center justify-self-end rounded-md border border-border bg-card text-foreground outline-none transition-colors hover:border-foreground/30 hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring md:hidden"
          aria-label={menuOpen ? "Close menu" : "Open menu"}
          aria-expanded={menuOpen}
          aria-controls="mobile-nav"
          onClick={() => setMenuOpen((v) => !v)}
        >
          {menuOpen ? <X aria-hidden size={20} /> : <Menu aria-hidden size={20} />}
        </button>
      </div>

      {menuOpen ? (
        <nav
          id="mobile-nav"
          aria-label="Site"
          className="border-t border-border bg-card md:hidden"
        >
          <div className="mx-auto flex max-w-6xl flex-col gap-4 px-3 py-3 sm:px-4">
            <ThemeSelect layout="panel" onChosen={closeMenu} />
            <AccountActions stacked onNavigated={closeMenu} />
            <Link
              to="/status"
              preload="intent"
              className="flex h-11 items-center justify-center rounded-md border border-border font-medium text-foreground no-underline outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring"
              activeProps={{
                className:
                  "flex h-11 items-center justify-center rounded-md border border-primary font-medium text-primary no-underline outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
              }}
              onClick={closeMenu}
            >
              Status
            </Link>
          </div>
        </nav>
      ) : null}
    </header>
  );
}

export function SiteFooter() {
  return (
    <footer className="border-t border-border bg-card/60 py-8 text-[14px] text-muted-foreground">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-4">
        <span>© Octanest</span>
        <Link
          to="/status"
          preload="intent"
          className="font-medium text-foreground underline-offset-4 hover:text-primary hover:underline"
          activeProps={{
            className:
              "font-medium text-primary underline underline-offset-4",
          }}
        >
          Status
        </Link>
      </div>
    </footer>
  );
}
