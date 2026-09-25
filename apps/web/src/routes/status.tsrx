import { createFileRoute, useLoaderData } from "@octanejs/tanstack-router";
import type { HealthResponse, RpcResult } from "@oxidean/api-client";
import { StatusPageSkeleton } from "@/components/page-skeletons";
import { fetchSystemHealth } from "@/lib/ssr-auth";

export type StatusPhase =
  | { kind: "healthy"; version: string; database: string }
  | { kind: "unhealthy"; message: string }
  | { kind: "unreachable"; message: string };

const UNREACHABLE: StatusPhase = {
  kind: "unreachable",
  message:
    "Oxidean couldn’t complete a health check. Confirm the stack is up (`docker compose` / `make dev`) and try again.",
};

const UNHEALTHY: StatusPhase = {
  kind: "unhealthy",
  message: "The API reported an unhealthy state. Retry in a moment or check your Compose/API logs.",
};

/** Map `system.health` RPC result → status UI phase (exported for tests). */
export function resolveStatusPhase(res: RpcResult<HealthResponse>): StatusPhase {
  if (!res.ok) return UNREACHABLE;
  if (res.data.status === "ok") {
    return {
      kind: "healthy",
      version: res.data.version,
      database: res.data.database,
    };
  }
  return UNHEALTHY;
}

export const Route = createFileRoute("/status")({
  component: StatusPage,
  head: () => ({ meta: [{ title: "Status · Oxidean" }] }),
  loader: async (): Promise<StatusPhase> => {
    try {
      return resolveStatusPhase(await fetchSystemHealth());
    } catch {
      return UNREACHABLE;
    }
  },
});

/** Exported for integration tests. */
export function StatusPage() @{
  const phase = useLoaderData({ from: "/status" }) as StatusPhase | undefined;

  @if (!phase) {
    <StatusPageSkeleton />;
  } @else {
    <StatusResolved phase={phase} />;
  }
}

/** Exported for integration tests — paints a resolved status phase. */
export function StatusResolved({ phase }: { phase: StatusPhase }) @{
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
            API health check succeeded. This page reflects live `system.health` — history
            arrives in a later release.
          </p>
          <p className="mt-4 text-[14px] text-muted-foreground">
            {"version " + phase.version + " · database " + phase.database}
          </p>
        </div>;
      }
      @if (phase.kind === "unhealthy") {
        <div className="oct-status-resolve">
          <p className="font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-destructive">
            Degraded or failing
          </p>
          <p className="mt-2 text-[16px] text-muted-foreground">{phase.message as string}</p>
        </div>;
      }
      @if (phase.kind === "unreachable") {
        <div className="oct-status-resolve">
          <p className="font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-destructive">
            Can’t reach the API
          </p>
          <p className="mt-2 text-[16px] text-muted-foreground">{phase.message as string}</p>
          <p className="mt-2 text-[14px] text-muted-foreground">
            Something blocked this request. Refresh the page or verify the API is running.
          </p>
        </div>;
      }
    </div>
  </div>;
}
