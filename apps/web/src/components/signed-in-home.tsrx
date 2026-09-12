import { Link } from "@octanejs/tanstack-router";
import type { UserPublic } from "@octanest/api-client";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/** Signed-in home content rendered at `/` (Gitea-style — no separate /dashboard). */
export function SignedInHome({ user }: { user: UserPublic }) @{
  const greetName =
    user.display_name?.trim() ||
    (user.username ? `@${user.username}` : "there");

  <div className="mx-auto max-w-3xl px-4 py-16">
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

    <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
      {"Welcome, " + greetName}
    </h1>
    <p className="mt-2 text-[16px] text-muted-foreground">
      You’re signed in. Profile and admin tools live here until a fuller home
      arrives.
    </p>

    <div className="mt-8 flex flex-wrap items-start gap-3">
      <div className="flex max-w-xs flex-col gap-2">
        <button
          type="button"
          disabled
          aria-disabled="true"
          title={
            user.email_verified
              ? "Repository creation arrives in a later phase."
              : "Verify your email to create a repository."
          }
          className={cn(buttonVariants({ variant: "default" }))}
        >
          New repository
        </button>
        <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
          @if (user.email_verified) {
            <>Repository creation arrives in a later phase.</>
          } @else {
            <>Verify your email to create a repository.</>
          }
        </p>
      </div>
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
