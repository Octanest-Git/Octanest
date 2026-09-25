import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { apiClient } from "@/lib/api-client";
import { toastSuccess } from "@/lib/toast";

const NETWORK_ERROR = "Can't reach Oxidean. Check your connection and try again.";

function enabledQueryKey(owner: string, name: string) {
  return ["repo", "templates", "getEnabled", owner, name] as const;
}

/** Repo Settings — expose repository as a create-from template (issue #18). */
export function TemplateRepoSettingsPanel(props: {
  owner: string;
  name: string;
  can_admin: boolean;
}) @{
  const owner = props.owner;
  const name = props.name;
  const canAdmin = props.can_admin;
  const queryClient = useQueryClient();

  const enabledQ = useQuery({
    queryKey: enabledQueryKey(owner, name),
    queryFn: async () => {
      const res = await apiClient.repo.templates.getEnabled({ owner, name });
      if (!res.ok) throw new Error(res.error.message || "Could not load template status.");
      return res.data.enabled;
    },
  });

  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");

  async function toggleEnabled(next: boolean) {
    if (!canAdmin) return;
    setError("");
    setPending(true);
    try {
      const res = await apiClient.repo.templates.setEnabled({
        owner,
        name,
        enabled: next,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not update template setting.");
        setPending(false);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: enabledQueryKey(owner, name) });
      toastSuccess(next ? "Repository marked as a template." : "Template flag cleared.");
      setPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  const enabled = enabledQ.data ?? false;

  <section className="mt-12 flex flex-col gap-4" aria-labelledby="template-repo-heading">
    <div>
      <h2
        id="template-repo-heading"
        className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-foreground"
      >
        Template repository
      </h2>
      <p className="mt-1 text-[14px] text-muted-foreground">
        Let others create a new repository from this repo&apos;s default branch tip (fresh initial
        commit — not a fork). Only people who can read this repository will see it on /new.
      </p>
    </div>
    <div className="flex items-center justify-between gap-4">
      <Label htmlFor="repo-template-enabled" className="text-[14px]">
        Publish as template
      </Label>
      <Switch
        id="repo-template-enabled"
        data-testid="repo-template-enabled"
        checked={enabled}
        disabled={!canAdmin || pending || enabledQ.isLoading}
        onCheckedChange={(v) => void toggleEnabled(!!v)}
      />
    </div>
    @if (error) {
      <p className="text-[14px] text-destructive" role="alert">
        {error as string}
      </p>;
    }
  </section>;
}
