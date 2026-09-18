import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { ArrowLeft, Workflow } from "@octanejs/lucide";
import { Badge } from "@/components/ui/badge";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";
import { PageHeader } from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { actionsJobLogQuery, actionsRunDetailQuery } from "@/lib/actions-queries";

/** Map a workflow run/job status to a status Badge variant. */
function runStatusVariant(status: string): "success" | "destructive" | "primary" | "default" {
  const s = status.toLowerCase();
  if (s === "success" || s === "completed" || s === "passed") return "success";
  if (s === "failure" || s === "failed" || s === "error" || s === "cancelled") return "destructive";
  if (s === "in_progress" || s === "running" || s === "queued" || s === "pending") return "primary";
  return "default";
}

export const Route = createFileRoute("/$owner/$repo/actions/$run")({
  component: RepoActionsRunPage,
  head: ({ params }) => {
    const p = params as { owner?: string; repo?: string; run?: string };
    return {
      meta: [
        {
          title: `Run ${String(p.run ?? "")} · ${String(p.owner ?? "")}/${String(p.repo ?? "")} · Octanest`,
        },
      ],
    };
  },
});

/** Run detail + job logs (ACT-03 / D-ACT-13). Inherits layout chrome (D-QH-01). */
export function RepoActionsRunPage() @{
  const params = useParams({ from: "/$owner/$repo/actions/$run" }) as {
    owner?: string;
    repo?: string;
    run?: string;
  };
  const owner = String(params.owner ?? "");
  const repoName = String(params.repo ?? "");
  const runId = String(params.run ?? "");
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);

  const detailQ = useQuery(actionsRunDetailQuery(owner, repoName, runId));
  const jobs = detailQ.data?.jobs ?? [];
  const run = detailQ.data?.run;
  const jobId = selectedJobId ?? jobs[0]?.id ?? "";
  const logQ = useQuery({
    ...actionsJobLogQuery(owner, repoName, runId, jobId),
    enabled: !!jobId,
  });
  const logText = logQ.data?.content || (logQ.isPending ? "Loading log…" : "(empty log)");
  const runDescription = run ? `${run.event} · ${run.head_sha.slice(0, 7)}` : "";

  @{
    <div data-testid="repo-actions-run">
      <p className="mb-4">
        <a
          href={`/${owner}/${repoName}/actions` as string}
          className="inline-flex items-center gap-1 text-[13px] text-muted-foreground no-underline hover:text-foreground hover:underline"
        >
          <ArrowLeft aria-hidden size={14} />
          Actions
        </a>
      </p>
      @if (detailQ.isPending) {
        <>
          <div className="flex flex-col gap-2 border-b border-border pb-4">
            <Skeleton className="h-6 w-1/2" />
            <Skeleton className="h-4 w-40" />
          </div>
          <Skeleton className="mt-8 h-64 w-full" />
        </>;
      } @else {
        @if (!run) {
          <EmptyState
            icon={<Workflow aria-hidden size={22} />}
            title="Run not found"
            description="This workflow run doesn't exist or is no longer available."
          />;
        } @else {
          <>
            <PageHeader
              icon={<Workflow aria-hidden size={22} />}
              title={run.title}
              description={runDescription}
              actions={
                <Badge variant={runStatusVariant(run.status)}>{run.status as string}</Badge>
              }
            />
            <div className="mt-8 grid gap-6 md:grid-cols-[16rem_1fr]">
              <List data-testid="repo-actions-jobs" className="self-start">
                @for (const job of jobs; key job.id) {
                  <ListRow className="p-0">
                    <button
                      type="button"
                      className={cn(
                        "flex w-full items-center justify-between gap-2 px-3 py-2 text-left text-[14px] outline-none transition-colors hover:bg-muted/40 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring",
                        job.id === jobId
                          ? "bg-muted/60 font-medium text-foreground"
                          : "text-foreground",
                      )}
                      onClick={() => setSelectedJobId(job.id)}
                    >
                      <span className="min-w-0 truncate">{job.name as string}</span>
                      <Badge variant={runStatusVariant(job.status)}>
                        {job.status as string}
                      </Badge>
                    </button>
                  </ListRow>;
                } @empty {
                  <ListRow>
                    <span className="text-[14px] text-muted-foreground">No jobs.</span>
                  </ListRow>;
                }
              </List>
              <pre
                data-testid="repo-actions-job-log"
                className="max-h-[28rem] overflow-auto rounded-md border border-border bg-muted/40 p-4 font-mono text-[12px] text-foreground"
              >
                {logText as string}
              </pre>
            </div>
          </>;
        }
      }
    </div>;
    };
}
