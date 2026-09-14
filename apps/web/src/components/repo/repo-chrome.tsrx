import { Lock } from "@octanejs/lucide";
import type { RepoPublic } from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { useChromeAccountState } from "@/lib/use-chrome-account";

type RepoChromeProps = {
  repo: RepoPublic;
  active:
    | "code"
    | "commits"
    | "branches"
    | "tags"
    | "issues"
    | "releases"
    | "settings";
  /** Force Settings nav (e.g. settings page). Owners also see it via session. */
  showSettings?: boolean;
};

/** Title row + Code/Commits/Branches/Tags/Issues/Releases/(Settings) link-row. */
export function RepoChrome({
  repo,
  active,
  showSettings = false,
}: RepoChromeProps) @{
  const { user } = useChromeAccountState();
  const isOwner =
    !!user?.username && user.username === repo.owner_username;
  const settingsVisible = showSettings || isOwner;

  const owner = repo.owner_username;
  const name = repo.name;
  const base = `/${owner}/${name}`;
  const codeHref = base;
  const commitsHref = `${base}/commits/${encodeURIComponent(repo.default_branch)}`;
  const branchesHref = `${base}/branches`;
  const tagsHref = `${base}/tags`;
  const issuesHref = `${base}/issues`;
  const releasesHref = `${base}/releases`;
  const settingsHref = `${base}/settings`;

  function navClass(isActive: boolean): string {
    return isActive
      ? "border-b-2 border-primary pb-2 text-[14px] font-semibold text-foreground no-underline"
      : "pb-2 text-[14px] font-normal text-muted-foreground no-underline hover:text-foreground";
  }

  <div className="mb-6">
    <div className="flex flex-wrap items-center gap-3">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        <a href="/" className="text-primary no-underline hover:underline">
          {owner as string}
        </a>
        <span className="mx-1 text-muted-foreground">/</span>
        <span>{name as string}</span>
      </h1>
      <Badge variant="outline">
        @if (repo.visibility === "private") {
          <Lock aria-hidden size={12} />
        }
        {(repo.visibility === "private" ? "Private" : "Public") as string}
      </Badge>
    </div>
    @if (repo.description.trim()) {
      <p className="mt-2 text-[16px] text-muted-foreground">
        {repo.description as string}
      </p>
    }

    <nav
      className="mt-6 flex flex-wrap gap-4 border-b border-border pb-0"
      aria-label="Repository"
    >
      <a
        href={codeHref as string}
        className={navClass(active === "code") as string}
        aria-current={active === "code" ? "page" : undefined}
      >
        Code
      </a>
      <a
        href={commitsHref as string}
        className={navClass(active === "commits") as string}
        aria-current={active === "commits" ? "page" : undefined}
      >
        Commits
      </a>
      <a
        href={branchesHref as string}
        className={navClass(active === "branches") as string}
        aria-current={active === "branches" ? "page" : undefined}
      >
        Branches
      </a>
      <a
        href={tagsHref as string}
        className={navClass(active === "tags") as string}
        aria-current={active === "tags" ? "page" : undefined}
      >
        Tags
      </a>
      <a
        href={issuesHref as string}
        className={navClass(active === "issues") as string}
        aria-current={active === "issues" ? "page" : undefined}
      >
        Issues
      </a>
      <a
        href={releasesHref as string}
        className={navClass(active === "releases") as string}
        aria-current={active === "releases" ? "page" : undefined}
      >
        Releases
      </a>
      @if (settingsVisible) {
        <a
          href={settingsHref as string}
          className={navClass(active === "settings") as string}
          aria-current={active === "settings" ? "page" : undefined}
        >
          Settings
        </a>
      }
    </nav>
  </div>
}
