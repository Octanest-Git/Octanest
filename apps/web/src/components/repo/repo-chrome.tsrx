import { Lock, Star } from "@octanejs/lucide";
import { useState } from "octane";
import type { RepoPublic } from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { RepoSearchEntry } from "@/components/repo/repo-search-entry";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";
import type { RepoChromeActive } from "@/lib/repo-chrome-active";
import { useChromeAccountState } from "@/lib/use-chrome-account";

type RepoChromeProps = {
  repo: RepoPublic;
  active: RepoChromeActive;
  /** Force Settings nav (e.g. settings page). Admins also see it via can_admin. */
  showSettings?: boolean;
};

/** Title row + Code/Commits/Branches/Tags/Issues/Releases/Packages/(Settings) link-row. */
export function RepoChrome({ repo, active, showSettings = false }: RepoChromeProps) @{
  // Gate Settings on API can_admin — not personal username equality (D-REL-07 / D-REL-09).
  const settingsVisible = showSettings || !!repo.can_admin;
  const account = useChromeAccountState();
  const [starCount, setStarCount] = useState(repo.star_count ?? 0);
  const [starred, setStarred] = useState(!!repo.viewer_has_starred);
  const [starBusy, setStarBusy] = useState(false);

  const owner = repo.owner_username;
  const name = repo.name;
  const base = `/${owner}/${name}`;
  const codeHref = base;
  const commitsHref = `${base}/commits/${encodeURIComponent(repo.default_branch)}`;
  const branchesHref = `${base}/branches`;
  const tagsHref = `${base}/tags`;
  const issuesHref = `${base}/issues`;
  const pullsHref = `${base}/pulls`;
  const releasesHref = `${base}/releases`;
  const packagesHref = `${base}/packages`;
  const actionsHref = `${base}/actions`;
  const settingsHref = `${base}/settings`;
  const forkHref = `${base}/fork`;
  const signedIn = !!account.user;

  async function toggleStar() {
    if (!signedIn || starBusy) return;
    setStarBusy(true);
    try {
      const res = starred
        ? await apiClient.repo.unstar({ owner, name })
        : await apiClient.repo.star({ owner, name });
      if (res.ok && res.data) {
        setStarCount(res.data.star_count ?? 0);
        setStarred(!!res.data.viewer_has_starred);
      }
    } finally {
      setStarBusy(false);
    }
  }

  function navClass(isActive: boolean): string {
    return isActive
      ? "border-b-2 border-primary pb-2 text-[14px] font-semibold text-foreground no-underline"
      : "pb-2 text-[14px] font-normal text-muted-foreground no-underline hover:text-foreground";
  }

  <div className="mb-8">
    <div className="mb-4 flex flex-wrap items-center gap-3">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold tracking-tight leading-[1.2] text-foreground">
        <a href={"/" + owner} className="text-primary no-underline hover:underline">
          {owner as string}
        </a>
        <span className="mx-1 text-muted-foreground">/</span>
        <span>{name as string}</span>
      </h1>
      <Badge variant="outline">
        @if (repo.visibility === "private") {
          <Lock aria-hidden size={12} />;
        }
        {(repo.visibility === "private" ? "Private" : "Public") as string}
      </Badge>
      <div className="ml-auto flex flex-wrap items-center gap-2">
        <RepoSearchEntry owner={owner as string} repo={name as string} />
        @if (signedIn) {
          <Button
            type="button"
            variant="secondary"
            className="h-8 gap-1.5 px-2.5 text-[13px]"
            disabled={starBusy}
            aria-pressed={starred}
            onClick={() => void toggleStar()}
          >
            <Star aria-hidden size={14} className={starred ? "fill-current" : undefined} />
            <span>{(starred ? "Starred" : "Star") as string}</span>
            <span className="text-muted-foreground">{String(starCount)}</span>
          </Button>;
        } @else {
          <a
            href={("/login?next=" + encodeURIComponent(base)) as string}
            className="inline-flex items-center gap-1.5 rounded-md border border-border px-2.5 py-1.5 text-[13px] text-foreground no-underline hover:bg-muted"
          >
            <Star aria-hidden size={14} />
            <span>Star</span>
            <span className="text-muted-foreground">{String(starCount)}</span>
          </a>;
        }
        @if (signedIn && repo.visibility === "public") {
          <a
            href={forkHref as string}
            className="inline-flex items-center gap-1.5 rounded-md border border-border px-2.5 py-1.5 text-[13px] text-foreground no-underline hover:bg-muted"
          >
            Fork
          </a>;
        }
      </div>
    </div>
    @if (repo.forked_from) {
      <p className="mt-2 text-[14px] text-muted-foreground">
        Forked from{" "}
        <a
          href={("/" + repo.forked_from.owner + "/" + repo.forked_from.name) as string}
          className="text-primary no-underline hover:underline"
        >
          {(repo.forked_from.owner + "/" + repo.forked_from.name) as string}
        </a>
      </p>;
    }
    @if (repo.description.trim()) {
      <p className="mt-2 text-[16px] text-muted-foreground">{repo.description as string}</p>;
    }

    <nav className="mt-6 flex flex-wrap gap-4 border-b border-border pb-0" aria-label="Repository">
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
        href={pullsHref as string}
        className={navClass(active === "pulls") as string}
        aria-current={active === "pulls" ? "page" : undefined}
      >
        Pulls
      </a>
      <a
        href={releasesHref as string}
        className={navClass(active === "releases") as string}
        aria-current={active === "releases" ? "page" : undefined}
      >
        Releases
      </a>
      <a
        href={packagesHref as string}
        className={navClass(active === "packages") as string}
        aria-current={active === "packages" ? "page" : undefined}
      >
        Packages
      </a>
      <a
        href={actionsHref as string}
        className={navClass(active === "actions") as string}
        aria-current={active === "actions" ? "page" : undefined}
        data-testid="repo-chrome-actions"
      >
        Actions
      </a>
      @if (settingsVisible) {
        <a
          href={settingsHref as string}
          className={navClass(active === "settings") as string}
          aria-current={active === "settings" ? "page" : undefined}
        >
          Settings
        </a>;
      }
    </nav>
  </div>;
}
