import { useQuery } from "@octanejs/tanstack-query";
import type { PullCommitSummary } from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";

export type PullCommitsProps = {
  owner: string;
  repoName: string;
  number: number;
};

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

export function PullCommits(props: PullCommitsProps) @{
  const commitsQ = useQuery({
    queryKey: ["pull", "commits", props.owner, props.repoName, props.number] as const,
    queryFn: async () => {
      const res = await apiClient.pull.commits({
        owner: props.owner,
        name: props.repoName,
        number: props.number,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
  });

  const commits: PullCommitSummary[] = commitsQ.data?.commits ?? [];

  <div data-testid="pull-commits">
    @if (commitsQ.isPending) {
      <p className="text-sm text-muted-foreground">Loading commits…</p>;
    } @else {
      @if (commitsQ.isError) {
        <p className="text-sm text-destructive">Could not load commits.</p>;
      } @else {
        @if (commits.length === 0) {
          <p className="text-sm text-muted-foreground">No commits on this pull request.</p>;
        } @else {
          <ul className="divide-y divide-border rounded-md border border-border">
            @for (const c of commits; key c.sha) {
              <li className="flex flex-wrap items-baseline gap-3 px-3 py-2 text-sm">
                <code className="font-mono text-[13px] text-muted-foreground">
                  {c.short_sha as string}
                </code>
                <span className="font-medium">{c.subject as string}</span>
                <span className="text-muted-foreground">
                  {c.author_name as string} · {c.authored_at as string}
                </span>
              </li>;
            }
          </ul>;
        }
      }
    }
  </div>;
}
