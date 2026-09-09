import { createFileRoute } from "@octanejs/tanstack-router";
import { createClient, systemHealthQueryOptions } from "@octanest/api-client";
import { useEffect, useState } from "octane";

export const Route = createFileRoute("/status")({
  component: StatusPage,
});

const client = createClient({
  baseUrl: typeof window !== "undefined" ? window.location.origin : "http://127.0.0.1:8080",
  credentials: "include",
});

type Phase =
  | { kind: "loading" }
  | { kind: "healthy"; version: string; database: string }
  | { kind: "unhealthy"; message: string }
  | { kind: "unreachable"; message: string };

function StatusPage() {
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        // Prefer shared queryOptions shape (D-21) even without React Query host here.
        const opts = systemHealthQueryOptions(client);
        const data = await opts.queryFn();
        if (cancelled) return;
        if (data.status === "ok") {
          setPhase({
            kind: "healthy",
            version: data.version,
            database: data.database,
          });
        } else {
          setPhase({
            kind: "unhealthy",
            message:
              "The API reported an unhealthy state. Retry in a moment or check your Compose/API logs.",
          });
        }
      } catch (e) {
        if (cancelled) return;
        setPhase({
          kind: "unreachable",
          message:
            "Octanest couldn’t complete a health check. Confirm the stack is up (`docker compose` / `make dev`) and try again.",
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="mx-auto max-w-3xl px-4 py-16">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold">
        System status
      </h1>
      <div className="mt-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
        {phase.kind === "loading" && (
          <p className="text-[var(--color-muted)]">Checking Octanest services…</p>
        )}
        {phase.kind === "healthy" && (
          <div>
            <p className="text-[20px] font-semibold text-[var(--color-accent-cool)]">
              All systems operational
            </p>
            <p className="mt-2 text-[var(--color-muted)]">
              API health check succeeded. This page reflects live `system.health`
              — history arrives in a later release.
            </p>
            <p className="mt-4 text-[14px] text-[var(--color-muted)]">
              version {phase.version} · database {phase.database}
            </p>
          </div>
        )}
        {phase.kind === "unhealthy" && (
          <div>
            <p className="text-[20px] font-semibold text-[var(--color-destructive)]">
              Degraded or failing
            </p>
            <p className="mt-2 text-[var(--color-muted)]">{phase.message}</p>
          </div>
        )}
        {phase.kind === "unreachable" && (
          <div>
            <p className="text-[20px] font-semibold text-[var(--color-destructive)]">
              Can’t reach the API
            </p>
            <p className="mt-2 text-[var(--color-muted)]">{phase.message}</p>
            <p className="mt-2 text-[14px] text-[var(--color-muted)]">
              Something blocked this request. Refresh the page or verify the API is
              running.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
