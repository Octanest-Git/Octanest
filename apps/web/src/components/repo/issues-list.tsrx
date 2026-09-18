import { useForm } from "@octanejs/tanstack-form";
import { CircleCheck, CircleDot } from "@octanejs/lucide";
import type { IssuePublic, LabelPublic, RepoPublic } from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";
import { Skeleton } from "@/components/ui/skeleton";

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

export type IssueFilterDraft = {
  author: string;
  label: string;
  assignee: string;
  q: string;
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
  onStateChange: (next: IssueListStateFilter) => void;
  onApplyFilters: (draft: IssueFilterDraft) => void;
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
  onStateChange,
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

  const filterForm = useForm({
    defaultValues: {
      author: filters.author,
      label: filters.label,
      assignee: filters.assignee,
      q: filters.q,
    },
    onSubmit: ({ value }) => {
      onApplyFilters({
        author: value.author.trim(),
        label: value.label.trim(),
        assignee: value.assignee.trim(),
        q: value.q.trim(),
      });
    },
  });

  <div className="flex flex-col gap-4">
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div role="tablist" aria-label="Issue state" className="flex flex-wrap gap-1">
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
          </a>;
        }
        @if (canWrite) {
          <a
            href={`${base}/issues/new` as string}
            className="inline-flex h-11 items-center rounded-md bg-primary px-4 text-[14px] font-semibold text-primary-foreground no-underline hover:brightness-110"
          >
            New issue
          </a>;
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
        void filterForm.handleSubmit();
      }}
    >
      <filterForm.Field name="author">
        {(field) => (
          <label className="flex flex-col gap-1 text-[13px] text-muted-foreground">
            Author
            <input
              type="text"
              name="author"
              autocomplete="off"
              placeholder="username"
              value={field.state.value as string}
              className={fieldClass() as string}
              onBlur={field.handleBlur}
              onInput={(e) => field.handleChange((e.target as HTMLInputElement).value)}
            />
          </label>
        )}
      </filterForm.Field>
      <filterForm.Field name="label">
        {(field) => (
          <label className="flex flex-col gap-1 text-[13px] text-muted-foreground">
            Label
            <select
              name="label"
              aria-label="Label"
              value={field.state.value as string}
              className={fieldClass() as string}
              onChange={(e) => field.handleChange((e.target as HTMLSelectElement).value)}
            >
              <option value="">Any label</option>
              @for (const lab of labels; key lab.id) {
                <option value={lab.id as string}>{lab.name as string}</option>;
              }
            </select>
          </label>
        )}
      </filterForm.Field>
      <filterForm.Field name="assignee">
        {(field) => (
          <label className="flex flex-col gap-1 text-[13px] text-muted-foreground">
            Assignee
            <input
              type="text"
              name="assignee"
              autocomplete="off"
              placeholder="username"
              value={field.state.value as string}
              className={fieldClass() as string}
              onBlur={field.handleBlur}
              onInput={(e) => field.handleChange((e.target as HTMLInputElement).value)}
            />
          </label>
        )}
      </filterForm.Field>
      <filterForm.Field name="q">
        {(field) => (
          <label className="flex min-w-[12rem] flex-1 flex-col gap-1 text-[13px] text-muted-foreground">
            Search
            <input
              type="search"
              name="q"
              autocomplete="off"
              placeholder="Search title or body"
              value={field.state.value as string}
              className={fieldClass() as string}
              onBlur={field.handleBlur}
              onInput={(e) => field.handleChange((e.target as HTMLInputElement).value)}
            />
          </label>
        )}
      </filterForm.Field>
      <button
        type="submit"
        className="inline-flex h-10 items-center rounded-md border border-border bg-muted px-4 text-[14px] font-medium text-foreground hover:bg-muted/80"
      >
        Apply filters
      </button>
    </form>

    @if (pending) {
      <List>
        @for (const i of [0, 1, 2, 3, 4]; key i) {
          <ListRow className="items-start gap-3">
            <Skeleton className="mt-0.5 size-4 rounded-full" />
            <div className="flex flex-1 flex-col gap-2">
              <Skeleton className="h-4 w-2/3" />
              <Skeleton className="h-3 w-1/3" />
            </div>
          </ListRow>;
        }
      </List>;
    } @else {
      @if (issues.length === 0) {
        <EmptyState
          icon={<CircleDot aria-hidden size={22} />}
          title={emptyCopy}
          description={
            filters.state === "open"
              ? "Track bugs, ideas, and tasks for this repository here."
              : "Nothing to show for this filter."
          }
        />;
      } @else {
        <>
          <List>
            @for (const issue of issues; key issue.id) {
              <ListRow className="items-start gap-3 py-3">
                @if (issue.state === "open") {
                  <CircleDot
                    aria-hidden
                    size={16}
                    className="mt-0.5 shrink-0 text-success"
                  />;
                } @else {
                  <CircleCheck
                    aria-hidden
                    size={16}
                    className="mt-0.5 shrink-0 text-[#8250df] dark:text-[#a371f7]"
                  />;
                }
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <a
                      href={`${base}/issues/${issue.number}` as string}
                      className="text-[15px] font-semibold text-foreground no-underline hover:text-primary"
                    >
                      {issue.title as string}
                    </a>
                    @if (issue.state === "open") {
                      <Badge variant="success">Open</Badge>;
                    } @else {
                      <Badge variant="merged">Closed</Badge>;
                    }
                  </div>
                  <p className="mt-1 text-[12px] text-muted-foreground">
                    {`#${issue.number} · opened by @${issue.author_username}` as string}
                  </p>
                </div>
              </ListRow>;
            }
          </List>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-[13px] text-muted-foreground">
              {`${from}–${to} of ${total} issue${total === 1 ? "" : "s"}` as string}
            </p>
            <div
              className="flex items-center gap-2"
              role="navigation"
              aria-label="Issue list pagination"
            >
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={!canPrev}
                onClick={() => onPrevPage()}
              >
                Previous
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={!canNext}
                onClick={() => onNextPage()}
              >
                Next
              </Button>
            </div>
          </div>
        </>;
      }
    }
  </div>;
}
