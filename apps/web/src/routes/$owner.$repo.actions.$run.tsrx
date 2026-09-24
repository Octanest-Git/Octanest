import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useMutation, useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { ArrowLeft, Ban, RotateCcw, Workflow } from "@octanejs/lucide";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";
import { PageHeader } from "@/components/ui/page-header";
import { Skeleton } from "@/components/ui/skeleton";
import { useRepoStore } from "@/lib/repo-store";
import { formatRelative } from "@/lib/relative-time";
import { cn } from "@/lib/utils";
import {
  actionsCancelRunMutation,
  actionsJobLogQuery,
  actionsRerunRunMutation,
  actionsRunDetailQuery,
} from "@/lib/actions-queries";

/** Map a workflow run/job status to a status Badge variant. */
function runStatusVariant(status: string): "success" | "destructive" | "primary" | "default" {
  const s = status.toLowerCase();
  if (s === "success" || s === "completed" || s === "passed") return "success";
  if (s === "failure" || s === "failed" || s === "error" || s === "cancelled") return "destructive";
  if (s === "in_progress" || s === "running" || s === "queued" || s === "pending") return "primary";
  return "default";
}

/** Short human duration between two ISO timestamps ("1m 5s"). */
function jobDuration(started?: string, finished?: string): string {
  if (!started) return "";
  const start = Date.parse(started);
  const end = finished ? Date.parse(finished) : Date.now();
  if (!Number.isFinite(start) || !Number.isFinite(end) || end < start) return "";
  const secs = Math.floor((end - start) / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ${secs % 60}s`;
  return `${Math.floor(mins / 60)}h ${mins % 60}m`;
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
  const repo = useRepoStore((s) => s.repo);
  const canWrite = !!repo?.can_write;
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

  const qc = useQueryClient();
  const rerunM = useMutation({
    ...actionsRerunRunMutation(),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["repo", "actions"] });
    },
  });
  const cancelM = useMutation({
    ...actionsCancelRunMutation(),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["repo", "actions"] });
    },
  });
  const mutationErr =
    rerunM.isError || cancelM.isError
      ? String(rerunM.error?.message ?? cancelM.error?.message ?? "Action failed")
      : "";

  const runActive = run
    ? ["queued", "in_progress", "pending", "running"].includes(run.status)
    : false;
  const runDescription = run
    ? [
        run.workflow_name,
        run.event,
        run.head_ref.replace(/^refs\/heads\//, ""),
        run.head_sha.slice(0, 7),
        run.actor ? `by ${run.actor}` : "",
        formatRelative(run.created_at),
      ]
        .filter(Boolean)
        .join(" · ")
    : "";

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
                <div className="flex items-center gap-2">
                  <Badge variant={runStatusVariant(run.status)}>{run.status as string}</Badge>
                  {canWrite && runActive ? (
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      data-testid="actions-cancel-run"
                      disabled={cancelM.isPending}
                      onClick={() => cancelM.mutate({ owner, name: repoName, run_id: runId })}
                    >
                      <Ban aria-hidden size={14} />
                      Cancel run
                    </Button>
                  ) : undefined}
                  {canWrite && !runActive ? (
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      data-testid="actions-rerun-run"
                      disabled={rerunM.isPending}
                      onClick={() => rerunM.mutate({ owner, name: repoName, run_id: runId })}
                    >
                      <RotateCcw aria-hidden size={14} />
                      Re-run jobs
                    </Button>
                  ) : undefined}
                </div>
              }
            />
            @if (mutationErr) {
              <p className="mt-2 text-[13px] text-destructive" role="alert">
                {mutationErr as string}
              </p>;
            }
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
                      <span className="min-w-0">
                        <span className="block truncate">{job.name as string}</span>
                        @if (
                          jobDuration(job.started_at, job.finished_at)
                        ) {
                          <span className="block text-[11px] text-muted-foreground">
                            {
                              jobDuration(
                                job.started_at,
                                job.finished_at,
                              ) as string
                            }
                          </span>;
                        }
                      </span>
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
