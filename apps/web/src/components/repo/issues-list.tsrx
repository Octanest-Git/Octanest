import type { IssuePublic, RepoPublic } from "@octanest/api-client";

export type IssueListStateFilter = "open" | "closed" | "all";

type IssuesListProps = {
  owner: string;
  repoName: string;
  repo: RepoPublic;
  issues: IssuePublic[];
  total: number;
  state: IssueListStateFilter;
  onStateChange: (next: IssueListStateFilter) => void;
  pending?: boolean;
};

function tabClass(active: boolean): string {
  return active
    ? "rounded-md bg-muted px-3 py-1.5 text-[14px] font-semibold text-foreground"
    : "rounded-md px-3 py-1.5 text-[14px] font-normal text-muted-foreground hover:text-foreground";
}

/** Open/Closed/All filter + issue rows (D-ISS-16 / D-ISS-19). */
export function IssuesList({
  owner,
  repoName,
  repo,
  issues,
  total,
  state,
  onStateChange,
  pending = false,
}: IssuesListProps) @{
  const base = `/${owner}/${repoName}`;
  const canWrite = !!repo.can_write;
  const canAdmin = !!repo.can_admin;
  const emptyCopy =
    state === "closed"
      ? "No closed issues"
      : state === "all"
        ? "No issues"
        : "No open issues";

  <div className="flex flex-col gap-4">
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div
        role="tablist"
        aria-label="Issue state"
        className="flex flex-wrap gap-1"
      >
        <button
          type="button"
          role="tab"
          aria-selected={state === "open"}
          className={tabClass(state === "open") as string}
          onClick={() => onStateChange("open")}
        >
          Open
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={state === "closed"}
          className={tabClass(state === "closed") as string}
          onClick={() => onStateChange("closed")}
        >
          Closed
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={state === "all"}
          className={tabClass(state === "all") as string}
          onClick={() => onStateChange("all")}
        >
          All
        </button>
      </div>
      <div className="flex flex-wrap items-center gap-3">
        @if (canAdmin) {
          <a
            href={`${base}/issues/labels` as string}
            className="text-[14px] font-medium text-primary no-underline hover:underline"
          >
            Labels
          </a>
        }
        @if (canWrite) {
          <a
            href={`${base}/issues/new` as string}
            className="inline-flex h-11 items-center rounded-md bg-primary px-4 text-[14px] font-semibold text-primary-foreground no-underline hover:brightness-110"
          >
            New issue
          </a>
        }
      </div>
    </div>

    @if (pending) {
      <p className="text-[16px] text-muted-foreground">Loading issues…</p>
    } @else {
      @if (issues.length === 0) {
        <p className="text-[16px] text-muted-foreground">{emptyCopy as string}</p>
      } @else {
        <>
          <ul className="divide-y divide-border border-y border-border">
            @for (const issue of issues; key issue.id) {
              <li className="flex flex-col gap-1 py-3">
                <a
                  href={`${base}/issues/${issue.number}` as string}
                  className="text-[16px] font-semibold text-foreground no-underline hover:text-primary"
                >
                  {issue.title as string}
                </a>
                <p className="text-[13px] text-muted-foreground">
                  {`#${issue.number} · ${issue.state} · @${issue.author_username}` as string}
                </p>
              </li>
            }
          </ul>
          <p className="text-[13px] text-muted-foreground">
            {`${total} issue${total === 1 ? "" : "s"}` as string}
          </p>
        </>
      }
    }
  </div>
}
