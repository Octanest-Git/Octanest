import type { PullListResponse } from "@octanest/api-client";

type PullsListProps = {
  owner: string;
  repo: string;
  list: PullListResponse;
};

export function PullsList(props: PullsListProps) @{
  const { owner, repo, list } = props;

  @if (list.pulls.length === 0) {
    <div className="rounded-md border border-dashed border-border p-8 text-center">
      <p className="text-muted-foreground">No pull requests match this filter.</p>
      <a
        href={`/${owner}/${repo}/pulls/new` as string}
        className="mt-3 inline-block text-sm text-primary no-underline hover:underline"
      >
        New pull request
      </a>
    </div>;
  } @else {
    <ul className="divide-y divide-border rounded-md border border-border">
      @for (const pull of list.pulls; key pull.id) {
        <li className="flex items-start justify-between gap-4 px-4 py-3">
          <div>
            <a
              href={`/${owner}/${repo}/pull/${pull.number}` as string}
              className="font-medium text-foreground no-underline hover:underline"
            >
              {pull.title as string}
            </a>
            <p className="mt-1 text-sm text-muted-foreground">
              #{pull.number as number} opened by {pull.author_username as string} ·{" "}
              {pull.base_ref as string} ← {pull.head_ref as string}
            </p>
          </div>
          <span className="shrink-0 text-xs uppercase text-muted-foreground">
            {pull.state as string}
          </span>
        </li>;
      }
    </ul>;
  }
}
