import { Menu, X } from "@octanejs/lucide";
import { Link, useNavigate } from "@octanejs/tanstack-router";
import { useQueryClient } from "@octanejs/tanstack-query";
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
import { clearSessionQueries } from "@/lib/session-queries";
import { syncSessionPresenceHint } from "@/lib/session-hint";
import { useChromeAccountState } from "@/lib/use-chrome-account";
import { cn } from "@/lib/utils";
import { Skeleton } from "@/components/ui/skeleton";

function initialsFor(user: UserPublic): string {
  const source = user.display_name?.trim() || user.username || "?";
  const parts = source.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return `${parts[0]![0] ?? ""}${parts[1]![0] ?? ""}`.toUpperCase();
  }
  return source.slice(0, 2).toUpperCase();
}

function AccountAvatar({
  user,
  size = "sm",
}: {
  user: UserPublic;
  size?: "sm" | "md";
}) {
  const box = size === "md" ? "size-9" : "size-7";
  const text = size === "md" ? "text-[12px]" : "text-[11px]";
  if (user.avatar_url) {
    return (
      <img
        src={user.avatar_url}
        alt=""
        width={size === "md" ? 36 : 28}
        height={size === "md" ? 36 : 28}
        className={cn("octanest-squircle shrink-0 object-cover", box)}
      />
    );
  }
  return (
    <span
      aria-hidden
      className={cn(
        "octanest-squircle inline-flex shrink-0 items-center justify-center bg-muted font-semibold text-foreground",
        box,
        text,
      )}
    >
      {initialsFor(user)}
    </span>
  );
}

function AccountMenu({
  user,
  onNavigated,
}: {
  user: UserPublic;
  onNavigated?: () => void;
}) @{
  const queryClient = useQueryClient();

  async function logout() {
    try {
      await apiClient.auth.logout();
    } catch {
      /* still clear local chrome */
    }
    syncSessionPresenceHint(false);
    clearSessionQueries(queryClient);
    onNavigated?.();
    window.location.assign("/");
  }

  <DropdownMenu>
    <DropdownMenuTrigger
      className={cn(
        "inline-flex h-11 items-center gap-2 rounded-md px-2 text-[14px] font-normal text-foreground outline-none",
        "hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
      )}
      aria-label="Account menu"
    >
      <AccountAvatar user={user} />
      <span className="hidden text-muted-foreground md:inline">
        {"@" + user.username}
      </span>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" className="min-w-48">
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
          window.location.assign("/settings/tokens");
        }}
      >
        Personal access tokens
      </DropdownMenuItem>
      <DropdownMenuItem
        onClick={() => {
          onNavigated?.();
          window.location.assign("/settings/ssh-keys");
        }}
      >
        SSH keys
      </DropdownMenuItem>
      @if (user.role === "sys-admin") {
        <DropdownMenuItem
          onClick={() => {
            onNavigated?.();
            window.location.assign("/admin/auth");
          }}
        >
          Auth settings
        </DropdownMenuItem>
      }
      <DropdownMenuSeparator />
      <DropdownMenuItem onClick={() => void logout()}>Log out</DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
}

function AccountActions({
  className,
  onNavigated,
}: {
  className?: string;
  onNavigated?: () => void;
}) {
  const { pending, user, needsSetup, allowSignup } = useChromeAccountState();

  if (pending || user === undefined || needsSetup === null) {
    return (
      <div
        role="group"
        aria-label="Account"
        aria-busy="true"
        className={cn(
          "flex h-11 w-[11.5rem] shrink-0 items-center justify-end gap-2",
          className,
        )}
      >
        <Skeleton className="h-11 w-[4.75rem]" />
        <Skeleton className="h-11 w-[4.75rem]" />
      </div>
    );
  }

  // UI-SPEC: while needs_setup, omit Sign in / Sign up (mark + theme OK).
  if (needsSetup) {
    return (
      <div
        role="group"
        aria-label="Account"
        className={cn(
          "flex h-11 w-[11.5rem] shrink-0 items-center justify-end gap-2",
          className,
        )}
      />
    );
  }

  if (user) {
    return (
      <div
        role="group"
        aria-label="Account"
        className={cn(
          "flex h-11 w-[11.5rem] shrink-0 items-center justify-end gap-2",
          className,
        )}
      >
        <AccountMenu user={user} onNavigated={onNavigated} />
      </div>
    );
  }

  return (
    <div
      role="group"
      aria-label="Account"
      className={cn(
        "flex h-11 w-[11.5rem] shrink-0 items-center justify-end gap-2",
        className,
      )}
    >
      <Link
        to="/login"
        preload="intent"
        className={cn(buttonVariants({ variant: "ghost" }), "h-11 px-3")}
        onClick={onNavigated}
      >
        Sign in
      </Link>
      @if (allowSignup) {
        <Link
          to="/signup"
          preload="intent"
          className={cn(buttonVariants({ variant: "secondary" }), "h-11 px-3")}
          onClick={onNavigated}
        >
          Sign up
        </Link>
      }
    </div>
  );
}

const mobileNavItemClass =
  "flex h-11 w-full items-center rounded-md px-3 text-[14px] font-medium text-foreground no-underline outline-none transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring";

function MobileNavSheet({ onClose }: { onClose: () => void }) @{
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { pending, user, needsSetup, allowSignup } = useChromeAccountState();

  async function logout() {
    try {
      await apiClient.auth.logout();
    } catch {
      /* still clear */
    }
    syncSessionPresenceHint(false);
    clearSessionQueries(queryClient);
    onClose();
    window.location.assign("/");
  }

  <nav
    id="mobile-nav"
    aria-label="Site"
    className={cn(
      "fixed inset-x-0 top-14 z-40 max-h-[calc(100dvh-3.5rem)] overflow-y-auto",
      "border-b border-border bg-card shadow-lg md:hidden",
    )}
  >
    <div className="mx-auto flex max-w-6xl flex-col gap-3 px-3 py-3 sm:px-4">
      <ThemeSelect layout="panel" />

      @if (pending || user === undefined || needsSetup === null) {
        <div className="flex flex-col gap-2" aria-busy="true" aria-label="Account">
          <div className="flex items-center gap-3 px-3 py-2">
            <Skeleton className="size-9 rounded-md" />
            <div className="flex flex-1 flex-col gap-2">
              <Skeleton className="h-4 w-28" />
              <Skeleton className="h-3 w-20" />
            </div>
          </div>
          <Skeleton className="h-11 w-full" />
          <Skeleton className="h-11 w-full" />
          <Skeleton className="h-11 w-full" />
        </div>
      } @else {
        @if (user) {
          <div className="flex flex-col">
            <div className="mb-1 flex items-center gap-3 rounded-md px-3 py-2">
              <AccountAvatar user={user} size="md" />
              <div className="min-w-0 flex-1">
                <p className="truncate text-[14px] font-medium text-foreground">
                  {(user.display_name?.trim() || user.username) as string}
                </p>
                <p className="truncate text-[13px] text-muted-foreground">
                  {"@" + user.username}
                </p>
              </div>
            </div>
            <div className="flex flex-col gap-0.5" role="list">
              <button
                type="button"
                className={cn(mobileNavItemClass, "text-left")}
                onClick={() => {
                  onClose();
                  void navigate({ to: "/" });
                }}
              >
                Home
              </button>
              <a
                href="/settings/profile"
                className={mobileNavItemClass}
                onClick={onClose}
              >
                Profile
              </a>
              @if (user.role === "sys-admin") {
                <a
                  href="/admin/auth"
                  className={mobileNavItemClass}
                  onClick={onClose}
                >
                  Auth settings
                </a>
              }
              <Link
                to="/status"
                preload="intent"
                className={mobileNavItemClass}
                activeProps={{
                  className: cn(mobileNavItemClass, "text-primary"),
                }}
                onClick={onClose}
              >
                Status
              </Link>
              <div className="my-1 h-px bg-border" role="separator" />
              <button
                type="button"
                className={cn(
                  mobileNavItemClass,
                  "text-left text-muted-foreground hover:text-foreground",
                )}
                onClick={() => {
                  void logout();
                }}
              >
                Log out
              </button>
            </div>
          </div>
        } @else {
          <div className="flex flex-col gap-2">
            <Link
              to="/status"
              preload="intent"
              className={mobileNavItemClass}
              activeProps={{
                className: cn(mobileNavItemClass, "text-primary"),
              }}
              onClick={onClose}
            >
              Status
            </Link>
            @if (!needsSetup) {
              <div
                className={cn(
                  "grid gap-2 pt-1",
                  allowSignup ? "grid-cols-2" : "grid-cols-1",
                )}
              >
                <Link
                  to="/login"
                  preload="intent"
                  className={cn(
                    buttonVariants({ variant: "ghost" }),
                    "h-11 w-full justify-center",
                  )}
                  onClick={onClose}
                >
                  Sign in
                </Link>
                @if (allowSignup) {
                  <Link
                    to="/signup"
                    preload="intent"
                    className={cn(
                      buttonVariants({ variant: "secondary" }),
                      "h-11 w-full justify-center",
                    )}
                    onClick={onClose}
                  >
                    Sign up
                  </Link>
                }
              </div>
            }
          </div>
        }
      }
    </div>
  </nav>
}

export function SiteHeader() @{
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
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("pointerdown", onDocPointer);
      document.removeEventListener("keydown", onDocKey);
      document.body.style.overflow = prevOverflow;
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

  // Fixed overlay stays inside <header> so we don't insert siblings next to
  // view-transition-named <main> (that caused insertBefore DOM errors).
  <header
    data-site-header
    className="sticky top-0 z-50 border-b border-border bg-card"
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
        @if (menuOpen) {
          <X aria-hidden size={20} />
        } @else {
          <Menu aria-hidden size={20} />
        }
      </button>
    </div>

    @if (menuOpen) {
      <>
        <button
          type="button"
          aria-label="Dismiss menu"
          className="fixed inset-0 z-30 bg-background/70 backdrop-blur-[2px] md:hidden"
          onClick={closeMenu}
        />
        <MobileNavSheet onClose={closeMenu} />
      </>
    }
  </header>
}

export function SiteFooter() @{
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
}
