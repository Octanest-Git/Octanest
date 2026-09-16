import { createFileRoute } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { AdminSettingsLayout } from "@/components/settings/settings-shell";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";
import { toastError, toastSuccess } from "@/lib/toast";

export const Route = createFileRoute("/admin/runners")({
  component: AdminRunnersPage,
  head: () => ({ meta: [{ title: "Actions runners · Octanest" }] }),
});

/** Instance registration tokens for official / third-party runners (D-ACT-08). */
export function AdminRunnersPage() @{
  const [tokenOnce, setTokenOnce] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const runnersQ = useQuery({
    queryKey: ["admin", "actions", "listRunners"] as const,
    queryFn: async () => {
      const res = await apiClient.admin.actions.listRunners();
      if (!res.ok) throw new Error(res.error.message);
      return res.data.runners;
    },
  });

  async function createToken() {
    if (busy) return;
    setBusy(true);
    const res = await apiClient.admin.actions.createRegistrationToken();
    setBusy(false);
    if (!res.ok) {
      toastError(res.error.message ?? "Failed to create registration token");
      return;
    }
    setTokenOnce(res.data.token);
    toastSuccess("Copy this token now — it will not be shown again");
  }

  const runners = runnersQ.data ?? [];

  @{
    <AdminSettingsLayout active="runners" data-testid="admin-runners">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold text-foreground">
        Actions runners
      </h1>
      <p className="mt-2 text-muted-foreground">
        Octanest does not provide managed CI minutes. Operators register self-hosted runners
        (official octanest-runner image or compatible clients) with a registration token.
      </p>
      <div className="mt-8 space-y-4">
        <Button type="button" disabled={busy} onClick={() => void createToken()}>
          Create registration token
        </Button>
        @if (tokenOnce) {
          <pre
            className="overflow-x-auto rounded-md bg-muted/40 p-4 font-mono text-[13px] text-foreground"
            data-testid="admin-runner-token-once"
          >
            {tokenOnce as string}
          </pre>;
        } @else {
          <p className="text-[14px] text-muted-foreground">
            Newly created tokens are shown once. Rotate compromised tokens immediately.
          </p>;
        }
      </div>
      <h2 className="mt-10 text-[18px] font-semibold text-foreground">Registered runners</h2>
      <ul className="mt-4 space-y-2">
        @for (const r of runners; key r.id) {
          <li className="text-[14px] text-foreground">
            {(r.name as string) + " — " + (r.labels.join(", ") as string)}
          </li>;
        } @empty {
          <li className="text-muted-foreground">No runners registered yet.</li>;
        }
      </ul>
    </AdminSettingsLayout>;
    };
}
