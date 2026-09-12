import { createFileRoute } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { systemHealthQueryOptions } from "@octanest/api-client";
import { StatusPageSkeleton } from "@/components/page-skeletons";
import { apiClient } from "@/lib/api-client";

export const Route = createFileRoute("/status")({
  component: StatusPage,
  head: () => ({ meta: [{ title: "Status · Octanest" }] }),
});

type Phase =
  | { kind: "healthy"; version: string; database: string }
  | { kind: "unhealthy"; message: string }
  | { kind: "unreachable"; message: string };

const UNREACHABLE: Phase = {
  kind: "unreachable",
  message:
    "Octanest couldn’t complete a health check. Confirm the stack is up (`docker compose` / `make dev`) and try again.",
};

const UNHEALTHY: Phase = {
  kind: "unhealthy",
  message:
    "The API reported an unhealthy state. Retry in a moment or check your Compose/API logs.",
};

/** Exported for integration tests. */
export function StatusPage() @{
  const health = useQuery(systemHealthQueryOptions(apiClient));

  let phase: Phase | null = null;
  if (health.isPending) {
    phase = null;
  } else if (health.isError || !health.data) {
    phase = UNREACHABLE;
  } else if (health.data.status === "ok") {
    phase = {
      kind: "healthy",
      version: health.data.version,
      database: health.data.database,
    };
  } else {
    phase = UNHEALTHY;
  }

  @if (!phase) {
    <StatusPageSkeleton />
  } @else {
    <StatusResolved phase={phase} />
  }
}

function StatusResolved({ phase }: { phase: Phase }) @{
  <div className="mx-auto max-w-3xl px-4 py-16">
    <style>
      @keyframes octStatusIn { from { opacity: 0; } to { opacity: 1; } }
      .oct-status-resolve { animation: octStatusIn 200ms ease-out both; }
      @media (prefers-reduced-motion: reduce) {
        .oct-status-resolve { animation: none !important; opacity: 1 !important; }
      }
    </style>
    <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold">
      System status
    </h1>
    <div className="mt-8 rounded-lg border border-border bg-card p-6">
      @if (phase.kind === "healthy") {
        <div className="oct-status-resolve">
          <p className="font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-primary">
            All systems operational
          </p>
          <p className="mt-2 text-[16px] text-muted-foreground">
            API health check succeeded. This page reflects live `system.health`
            — history arrives in a later release.
          </p>
          <p className="mt-4 text-[14px] text-muted-foreground">
            {"version " + phase.version + " · database " + phase.database}
          </p>
        </div>
      }
      @if (phase.kind === "unhealthy") {
        <div className="oct-status-resolve">
          <p className="font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-destructive">
            Degraded or failing
          </p>
          <p className="mt-2 text-[16px] text-muted-foreground">{phase.message as string}</p>
        </div>
      }
      @if (phase.kind === "unreachable") {
        <div className="oct-status-resolve">
          <p className="font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-destructive">
            Can’t reach the API
          </p>
          <p className="mt-2 text-[16px] text-muted-foreground">{phase.message as string}</p>
          <p className="mt-2 text-[14px] text-muted-foreground">
            Something blocked this request. Refresh the page or verify the API is
            running.
          </p>
        </div>
      }
    </div>
  </div>
}
