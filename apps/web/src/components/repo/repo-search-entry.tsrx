import { Search } from "@octanejs/lucide";

type RepoSearchEntryProps = {
  owner: string;
  repo: string;
  /** Optional prefilled query for deep links. */
  q?: string;
};

/** Compact chrome control → `/{owner}/{repo}/search` (D-SRCH-03). */
export function RepoSearchEntry({ owner, repo, q = "" }: RepoSearchEntryProps) @{
  const href =
    q.trim().length > 0
      ? `/${owner}/${repo}/search?q=${encodeURIComponent(q)}`
      : `/${owner}/${repo}/search`;

  <a
    href={href as string}
    className="inline-flex items-center gap-1.5 rounded-md border border-border px-2.5 py-1.5 text-[13px] text-muted-foreground no-underline hover:bg-muted/40 hover:text-foreground"
    aria-label="Search this repository"
    data-testid="repo-search-entry"
  >
    <Search aria-hidden size={14} />
    <span>Search</span>
  </a>;
}
