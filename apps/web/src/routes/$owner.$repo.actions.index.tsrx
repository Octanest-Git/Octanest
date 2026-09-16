import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { actionsRunsQuery } from "@/lib/actions-queries";

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
      <h2 className="font-[family-name:var(--font-display)] text-[20px] font-semibold text-foreground">
        Actions
      </h2>
      <p className="mt-1 text-muted-foreground">Workflow runs for this repository.</p>
      @if (err) {
        <p className="mt-8 text-destructive" role="alert">
          {err as string}
        </p>;
      } @else {
        @if (loading) {
          <p className="mt-8 text-muted-foreground">Loading runs…</p>;
        } @else {
          <ul className="mt-8 space-y-3" data-testid="repo-actions-run-list">
            @for (const run of runs; key run.id) {
              <li className="flex flex-wrap items-center gap-3 border-b border-border pb-3">
                <a
                  href={`/${owner}/${repoName}/actions/${run.id}` as string}
                  className="font-medium text-foreground underline-offset-2 hover:underline"
                >
                  {run.title as string}
                </a>
                <span className="rounded-sm bg-muted px-1.5 py-0.5 text-[12px] uppercase text-muted-foreground">
                  {run.status as string}
                </span>
                <span className="text-[12px] text-muted-foreground">
                  {run.event as string}
                </span>
                <span className="font-mono text-[12px] text-muted-foreground">
                  {run.head_sha.slice(0, 7) as string}
                </span>
              </li>;
            } @empty {
              <li className="text-muted-foreground">No workflow runs yet.</li>;
            }
          </ul>;
        }
      }
    </div>;
    };
}
