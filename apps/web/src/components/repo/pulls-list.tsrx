import { GitMerge, GitPullRequest, GitPullRequestClosed } from "@octanejs/lucide";
import type { PullListResponse } from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";

type PullsListProps = {
  owner: string;
  repo: string;
  list: PullListResponse;
};

export function PullsList(props: PullsListProps) @{
  const { owner, repo, list } = props;

  @if (list.pulls.length === 0) {
    <EmptyState
      icon={<GitPullRequest aria-hidden size={22} />}
      title="No pull requests"
      description="No pull requests match this filter. Open one to propose changes."
      action={
        <a href={`/${owner}/${repo}/pulls/new` as string} className="no-underline">
          <Button type="button" size="sm">
            New pull request
          </Button>
        </a>
      }
    />;
  } @else {
    <List>
      @for (const pull of list.pulls; key pull.id) {
        <ListRow className="items-start gap-3 py-3">
          @if (pull.state === "merged") {
            <GitMerge
              aria-hidden
              size={16}
              className="mt-0.5 shrink-0 text-[#8250df] dark:text-[#a371f7]"
            />;
          } @else if (pull.state === "closed") {
            <GitPullRequestClosed
              aria-hidden
              size={16}
              className="mt-0.5 shrink-0 text-destructive"
            />;
          } @else {
            <GitPullRequest
              aria-hidden
              size={16}
              className="mt-0.5 shrink-0 text-success"
            />;
          }
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <a
                href={`/${owner}/${repo}/pull/${pull.number}` as string}
                className="text-[15px] font-semibold text-foreground no-underline hover:text-primary"
              >
                {pull.title as string}
              </a>
              @if (pull.state === "merged") {
                <Badge variant="merged">Merged</Badge>;
              } @else if (pull.state === "closed") {
                <Badge variant="destructive">Closed</Badge>;
              } @else {
                <Badge variant="success">Open</Badge>;
              }
            </div>
            <p className="mt-1 text-[12px] text-muted-foreground">
              {`#${pull.number} opened by @${pull.author_username}` as string}
              <span className="mx-1.5">·</span>
              <span className="font-mono">{pull.base_ref as string}</span>
              <span className="mx-1"> ← </span>
              <span className="font-mono">{pull.head_ref as string}</span>
            </p>
          </div>
        </ListRow>;
      }
    </List>;
  }
}
