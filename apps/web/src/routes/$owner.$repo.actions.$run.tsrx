import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { actionsJobLogQuery, actionsRunDetailQuery } from "@/lib/actions-queries";

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
  const statusLine = run ? `${run.status} · ${run.event} · ${run.head_sha.slice(0, 7)}` : "";

  @{
    <div data-testid="repo-actions-run">
      <p className="mb-4">
        <a
          href={`/${owner}/${repoName}/actions` as string}
          className="text-muted-foreground underline-offset-2 hover:underline"
        >
          ← Actions
        </a>
      </p>
      @if (detailQ.isPending) {
        <p className="text-muted-foreground">Loading run…</p>;
      } @else {
        @if (!run) {
          <p className="text-muted-foreground">Run not found.</p>;
        } @else {
          <>
            <h2 className="font-[family-name:var(--font-display)] text-[20px] font-semibold text-foreground">
              {run.title as string}
            </h2>
            <p className="mt-1 text-muted-foreground">{statusLine as string}</p>
            <div className="mt-8 grid gap-6 md:grid-cols-[14rem_1fr]">
              <ul data-testid="repo-actions-jobs" className="space-y-2">
                @for (const job of jobs; key job.id) {
                  <li>
                    <button
                      type="button"
                      className="w-full text-left text-[14px] text-foreground hover:underline"
                      onClick={() => setSelectedJobId(job.id)}
                    >
                      {(job.name + " (" + job.status + ")") as string}
                    </button>
                  </li>;
                } @empty {
                  <li className="text-muted-foreground">No jobs.</li>;
                }
              </ul>
              <pre
                data-testid="repo-actions-job-log"
                className="max-h-[28rem] overflow-auto rounded-md bg-muted/40 p-4 font-mono text-[12px] text-foreground"
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
