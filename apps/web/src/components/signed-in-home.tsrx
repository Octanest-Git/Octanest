import { Link } from "@octanejs/tanstack-router";
import { Lock } from "@octanejs/lucide";
import type { RepoPublic, UserPublic } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { formatRelativeUpdated } from "@/lib/relative-time";
import { cn } from "@/lib/utils";

type ListPhase =
  | { kind: "loading" }
  | { kind: "ready"; repos: RepoPublic[] }
  | { kind: "error"; message: string };

/** Signed-in home dashboard — repo list + empty/activity states (D-13 / UI E1). */
export function SignedInHome({ user }: { user: UserPublic }) @{
  const greetName =
    user.display_name?.trim() ||
    (user.username ? `@${user.username}` : "there");
  const [list, setList] = useState<ListPhase>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await apiClient.repo.listMine();
        if (cancelled) return;
        if (!res.ok) {
          setList({
            kind: "error",
            message: res.error.message || "Could not load repositories.",
          });
          return;
        }
        setList({ kind: "ready", repos: res.data.repos });
      } catch {
        if (cancelled) return;
        setList({
          kind: "error",
          message: "Can't reach Octanest. Check your connection and try again.",
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const empty = list.kind === "ready" && list.repos.length === 0;

  <div className="mx-auto max-w-3xl px-4 py-16 md:px-4">
    @if (user.profile_incomplete) {
      <p
        role="status"
        className="mb-6 rounded-md border border-border bg-card px-4 py-3 text-[14px] text-foreground"
      >
        Choose a username to finish setup.{" "}
        <a
          href="/settings/profile"
          className="font-semibold text-primary underline-offset-4 hover:underline"
        >
          Profile
        </a>
      </p>
    }

    <div className="flex flex-wrap items-start justify-between gap-4">
      <div className="min-w-0">
        <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
          Your repositories
        </h1>
        <p className="mt-2 text-[16px] text-muted-foreground">
          {"Welcome, " + greetName + ". Recently updated first."}
        </p>
      </div>
      <div className="flex max-w-xs flex-col gap-2">
        @if (user.email_verified) {
          <Link
            to="/new"
            preload="intent"
            className={cn(buttonVariants({ variant: "default" }))}
          >
            New repository
          </Link>
        } @else {
          <>
            <button
              type="button"
              disabled
              aria-disabled="true"
              title="Verify your email to create a repository."
              className={cn(buttonVariants({ variant: "default" }))}
            >
              New repository
            </button>
            <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
              Verify your email to create a repository.
            </p>
          </>
        }
      </div>
    </div>

    <div className="mt-10 grid gap-8 md:grid-cols-[minmax(0,1fr)_14rem]">
      <section aria-label="Repository list" className="min-w-0">
        @if (list.kind === "loading") {
          <div className="flex flex-col gap-3" aria-busy="true">
            <Skeleton className="h-14 w-full" />
            <Skeleton className="h-14 w-full" />
            <Skeleton className="h-14 w-full" />
          </div>
        } @else {
          @if (list.kind === "error") {
            <p role="alert" className="text-[16px] text-destructive">
              {list.message as string}
            </p>
          } @else {
            @if (empty) {
              <div className="rounded-md border border-dashed border-border px-6 py-12 text-center">
                <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
                  Create your first repository
                </h2>
                <p className="mx-auto mt-3 max-w-md text-[16px] text-muted-foreground">
                  Host your code on Octanest — public or private, cloud or yours.
                </p>
                <div className="mt-6 flex justify-center">
                  @if (user.email_verified) {
                    <Link
                      to="/new"
                      preload="intent"
                      className={cn(buttonVariants({ variant: "default" }))}
                    >
                      New repository
                    </Link>
                  } @else {
                    <button
                      type="button"
                      disabled
                      aria-disabled="true"
                      title="Verify your email to create a repository."
                      className={cn(buttonVariants({ variant: "default" }))}
                    >
                      New repository
                    </button>
                  }
                </div>
              </div>
            } @else {
              <ul className="divide-y divide-border border-y border-border">
                @for (const repo of list.repos; key repo.id) {
                  <li className="flex flex-wrap items-center gap-3 py-4">
                    <a
                      href={"/" + repo.owner_username + "/" + repo.name}
                      className="min-w-0 flex-1 truncate text-[16px] font-semibold text-foreground no-underline hover:underline"
                    >
                      {repo.owner_username + "/" + repo.name}
                    </a>
                    <Badge variant="outline">
                      @if (repo.visibility === "private") {
                        <Lock aria-hidden size={12} />
                      }
                      {(repo.visibility === "private" ? "Private" : "Public") as string}
                    </Badge>
                    <span className="text-[14px] font-normal text-muted-foreground">
                      {formatRelativeUpdated(repo.updated_at) as string}
                    </span>
                  </li>
                }
              </ul>
            }
          }
        }
      </section>

      <aside
        aria-label="Activity"
        className="rounded-md border border-border bg-card/40 px-4 py-4"
      >
        <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
          Activity will show up here.
        </p>
      </aside>
    </div>

    <div className="mt-10 flex flex-wrap gap-3">
      <a
        href="/settings/profile"
        className={cn(buttonVariants({ variant: "secondary" }))}
      >
        Profile
      </a>
      <Link
        to="/status"
        preload="intent"
        className={cn(buttonVariants({ variant: "ghost" }))}
      >
        Status
      </Link>
      @if (user.role === "sys-admin") {
        <a
          href="/admin/auth"
          className={cn(buttonVariants({ variant: "ghost" }))}
        >
          Auth settings
        </a>
      }
    </div>
  </div>
}
