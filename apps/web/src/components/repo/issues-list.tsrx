import type { IssuePublic, LabelPublic, RepoPublic } from "@octanest/api-client";

export type IssueListStateFilter = "open" | "closed" | "all";

export type IssueListFilters = {
  state: IssueListStateFilter;
  author: string;
  label: string;
  assignee: string;
  q: string;
  offset: number;
  limit: number;
};

export const ISSUE_LIST_PAGE_SIZE = 25;

type IssuesListProps = {
  owner: string;
  repoName: string;
  repo: RepoPublic;
  issues: IssuePublic[];
  total: number;
  filters: IssueListFilters;
  labels: LabelPublic[];
  draftAuthor: string;
  draftLabel: string;
  draftAssignee: string;
  draftQ: string;
  onStateChange: (next: IssueListStateFilter) => void;
  onDraftAuthor: (next: string) => void;
  onDraftLabel: (next: string) => void;
  onDraftAssignee: (next: string) => void;
  onDraftQ: (next: string) => void;
  onApplyFilters: () => void;
  onPrevPage: () => void;
  onNextPage: () => void;
  pending?: boolean;
};

function tabClass(active: boolean): string {
  return active
    ? "rounded-md bg-muted px-3 py-1.5 text-[14px] font-semibold text-foreground"
    : "rounded-md px-3 py-1.5 text-[14px] font-normal text-muted-foreground hover:text-foreground";
}

function fieldClass(): string {
  return "h-10 rounded-md border border-border bg-background px-3 text-[14px] text-foreground";
}

/** Open/Closed/All + author/label/assignee/text filters + offset pages (D-ISS-16..19). */
export function IssuesList({
  owner,
  repoName,
  repo,
  issues,
  total,
  filters,
  labels,
  draftAuthor,
  draftLabel,
  draftAssignee,
  draftQ,
  onStateChange,
  onDraftAuthor,
  onDraftLabel,
  onDraftAssignee,
  onDraftQ,
  onApplyFilters,
  onPrevPage,
  onNextPage,
  pending = false,
}: IssuesListProps) @{
  const base = `/${owner}/${repoName}`;
  const canWrite = !!repo.can_write;
  const canAdmin = !!repo.can_admin;
  const emptyCopy =
    filters.state === "closed"
      ? "No closed issues"
      : filters.state === "all"
        ? "No issues"
        : "No open issues";
  const from = total === 0 ? 0 : filters.offset + 1;
  const to = Math.min(filters.offset + issues.length, total);
  const canPrev = filters.offset > 0;
  const canNext = filters.offset + filters.limit < total;

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
          aria-selected={filters.state === "open"}
          className={tabClass(filters.state === "open") as string}
          onClick={() => onStateChange("open")}
        >
          Open
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={filters.state === "closed"}
          className={tabClass(filters.state === "closed") as string}
          onClick={() => onStateChange("closed")}
        >
          Closed
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={filters.state === "all"}
          className={tabClass(filters.state === "all") as string}
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

    <form
      method="post"
      action="#"
      aria-label="Issue filters"
      className="flex flex-wrap items-end gap-3"
      onSubmit={(e) => {
        e.preventDefault();
        onApplyFilters();
      }}
    >
      <label className="flex flex-col gap-1 text-[13px] text-muted-foreground">
        Author
        <input
          type="text"
          name="author"
          autocomplete="off"
          placeholder="username"
          value={draftAuthor as string}
          className={fieldClass() as string}
          onInput={(e) => onDraftAuthor((e.target as HTMLInputElement).value)}
        />
      </label>
      <label className="flex flex-col gap-1 text-[13px] text-muted-foreground">
        Label
        <select
          name="label"
          aria-label="Label"
          value={draftLabel as string}
          className={fieldClass() as string}
          onChange={(e) =>
            onDraftLabel((e.target as HTMLSelectElement).value)
          }
        >
          <option value="">Any label</option>
          @for (const lab of labels; key lab.id) {
            <option value={lab.id as string}>{lab.name as string}</option>
          }
        </select>
      </label>
      <label className="flex flex-col gap-1 text-[13px] text-muted-foreground">
        Assignee
        <input
          type="text"
          name="assignee"
          autocomplete="off"
          placeholder="username"
          value={draftAssignee as string}
          className={fieldClass() as string}
          onInput={(e) =>
            onDraftAssignee((e.target as HTMLInputElement).value)
          }
        />
      </label>
      <label className="flex min-w-[12rem] flex-1 flex-col gap-1 text-[13px] text-muted-foreground">
        Search
        <input
          type="search"
          name="q"
          autocomplete="off"
          placeholder="Search title or body"
          value={draftQ as string}
          className={fieldClass() as string}
          onInput={(e) => onDraftQ((e.target as HTMLInputElement).value)}
        />
      </label>
      <button
        type="button"
        className="inline-flex h-10 items-center rounded-md border border-border bg-muted px-4 text-[14px] font-medium text-foreground hover:bg-muted/80"
        onClick={() => onApplyFilters()}
      >
        Apply filters
      </button>
    </form>

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
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-[13px] text-muted-foreground">
              {`${from}–${to} of ${total} issue${total === 1 ? "" : "s"}` as string}
            </p>
            <div
              className="flex items-center gap-2"
              role="navigation"
              aria-label="Issue list pagination"
            >
              <button
                type="button"
                disabled={!canPrev}
                className="inline-flex h-10 items-center rounded-md border border-border px-3 text-[14px] font-medium text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                onClick={() => onPrevPage()}
              >
                Previous
              </button>
              <button
                type="button"
                disabled={!canNext}
                className="inline-flex h-10 items-center rounded-md border border-border px-3 text-[14px] font-medium text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                onClick={() => onNextPage()}
              >
                Next
              </button>
            </div>
          </div>
        </>
      }
    }
  </div>
}
