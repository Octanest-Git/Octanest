import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { Workflow } from "@octanejs/lucide";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";
import { PageHeader } from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { actionsRunsQuery } from "@/lib/actions-queries";

/** Map a workflow run status to a status Badge variant. */
function runStatusVariant(status: string): "success" | "destructive" | "primary" | "default" {
  const s = status.toLowerCase();
  if (s === "success" || s === "completed" || s === "passed") return "success";
  if (s === "failure" || s === "failed" || s === "error" || s === "cancelled") return "destructive";
  if (s === "in_progress" || s === "running" || s === "queued" || s === "pending") return "primary";
  return "default";
}

export const Route = createFileRoute("/$owner/$repo/actions/")({
  component: RepoActionsPage,
  head: ({ params }) => {
    const p = params as { owner?: string; repo?: string };
    return {
      meta: [
        {
          title: `${String(p.owner ?? "")}/${String(p.repo ?? "")} actions · Octanest`,
        },
      ],
    };
  },
});

/** Workflow runs list (ACT-03 / D-ACT-12). Chrome from parent layout (D-QH-01). */
export function RepoActionsPage() @{
  const params = useParams({ from: "/$owner/$repo/actions/" }) as {
    owner?: string;
    repo?: string;
  };
  const owner = String(params.owner ?? "");
  const repoName = String(params.repo ?? "");
  const listQ = useQuery(actionsRunsQuery(owner, repoName));
  const runs = listQ.data?.runs ?? [];
  const loading = listQ.isPending;
  const err = listQ.isError ? String(listQ.error?.message ?? "Failed to load runs") : null;

  @{
    <div data-testid="repo-actions">
      <PageHeader
        icon={<Workflow aria-hidden size={22} />}
        title="Actions"
        description="Workflow runs for this repository."
      />
      @if (err) {
        <p className="mt-8 text-destructive" role="alert">
          {err as string}
        </p>;
      } @else {
        @if (loading) {
          <List className="mt-6">
            @for (const i of [0, 1, 2, 3]; key i) {
              <ListRow className="gap-3">
                <Skeleton className="h-4 flex-1" />
                <Skeleton className="h-5 w-16 rounded-md" />
              </ListRow>;
            }
          </List>;
        } @else {
          @if (runs.length === 0) {
            <EmptyState
              className="mt-6"
              icon={<Workflow aria-hidden size={22} />}
              title="No workflow runs yet"
              description="Runs triggered by pushes and other events will appear here."
            />;
          } @else {
            <List className="mt-6" data-testid="repo-actions-run-list">
              @for (const run of runs; key run.id) {
                <ListRow className="flex-wrap gap-3">
                  <a
                    href={`/${owner}/${repoName}/actions/${run.id}` as string}
                    className="min-w-0 flex-1 truncate font-medium text-foreground no-underline hover:text-primary hover:underline"
                  >
                    {run.title as string}
                  </a>
                  <Badge variant={runStatusVariant(run.status)}>
                    {run.status as string}
                  </Badge>
                  <span className="text-[12px] text-muted-foreground">
                    {run.event as string}
                  </span>
                  <span className="font-mono text-[12px] text-muted-foreground">
                    {run.head_sha.slice(0, 7) as string}
                  </span>
                </ListRow>;
              }
            </List>;
          }
        }
      }
    </div>;
    };
}
