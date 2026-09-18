import { useQuery } from "@octanejs/tanstack-query";
import { GitCommitHorizontal } from "@octanejs/lucide";
import type { PullCommitSummary } from "@octanest/api-client";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";
import { Skeleton } from "@/components/ui/skeleton";
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
      <List>
        @for (const i of [0, 1, 2]; key i) {
          <ListRow className="gap-3">
            <Skeleton className="h-4 w-14" />
            <Skeleton className="h-4 flex-1" />
          </ListRow>;
        }
      </List>;
    } @else {
      @if (commitsQ.isError) {
        <p role="alert" className="text-sm text-destructive">
          Could not load commits.
        </p>;
      } @else {
        @if (commits.length === 0) {
          <EmptyState
            compact
            icon={<GitCommitHorizontal aria-hidden size={20} />}
            title="No commits"
            description="This pull request doesn't contain any commits yet."
          />;
        } @else {
          <List>
            @for (const c of commits; key c.sha) {
              <ListRow className="flex-wrap items-baseline gap-3 text-sm">
                <code className="font-mono text-[13px] text-primary">
                  {c.short_sha as string}
                </code>
                <span className="min-w-0 flex-1 truncate font-medium">
                  {c.subject as string}
                </span>
                <span className="text-[12px] text-muted-foreground">
                  {c.author_name as string} · {c.authored_at as string}
                </span>
              </ListRow>;
            }
          </List>;
        }
      }
    }
  </div>;
}
