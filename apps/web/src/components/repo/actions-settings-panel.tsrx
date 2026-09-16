import type { ActionSecretMetaPublic } from "@octanest/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { apiClient } from "@/lib/api-client";
import { toastError, toastSuccess } from "@/lib/toast";

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

function secretsQueryKey(owner: string, name: string) {
  return ["repo", "actions", "secrets", owner, name] as const;
}

function enabledQueryKey(owner: string, name: string) {
  return ["repo", "actions", "getEnabled", owner, name] as const;
}

/** Repo Settings Actions enable + secrets CRUD (D-ACT-06 / D-ACT-17 / D-ACT-10). */
export function ActionsSettingsPanel(props: {
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
      const res = await apiClient.repo.actions.getEnabled({ owner, name });
      if (!res.ok) throw new Error(res.error.message || "Could not load Actions status.");
      return res.data.enabled;
    },
  });

  const secretsQ = useQuery({
    queryKey: secretsQueryKey(owner, name),
    queryFn: async (): Promise<ActionSecretMetaPublic[]> => {
      const res = await apiClient.repo.actions.secrets.list({ owner, name });
      if (!res.ok) throw new Error(res.error.message || "Could not load secrets.");
      return res.data.secrets;
    },
    enabled: canAdmin,
  });

  const [togglePending, setTogglePending] = useState(false);
  const [secretName, setSecretName] = useState("");
  const [secretValue, setSecretValue] = useState("");
  const [secretPending, setSecretPending] = useState(false);
  const [error, setError] = useState("");

  async function toggleEnabled(next: boolean) {
    if (!canAdmin) return;
    setError("");
    setTogglePending(true);
    try {
      const res = await apiClient.repo.actions.setEnabled({ owner, name, enabled: next });
      if (!res.ok) {
        setError(res.error.message || "Could not update Actions setting.");
        setTogglePending(false);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: enabledQueryKey(owner, name) });
      toastSuccess(next ? "Actions enabled." : "Actions disabled.");
      setTogglePending(false);
    } catch {
      setError(NETWORK_ERROR);
      setTogglePending(false);
    }
  }

  async function putSecret() {
    if (!canAdmin) return;
    const sn = secretName.trim();
    if (!sn || !secretValue) return;
    setError("");
    setSecretPending(true);
    try {
      const res = await apiClient.repo.actions.secrets.put({
        owner,
        name,
        secret_name: sn,
        value: secretValue,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not save secret.");
        setSecretPending(false);
        return;
      }
      setSecretName("");
      setSecretValue("");
      await queryClient.invalidateQueries({ queryKey: secretsQueryKey(owner, name) });
      toastSuccess("Secret saved. Value cannot be viewed again.");
      setSecretPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setSecretPending(false);
    }
  }

  async function deleteSecret(sn: string) {
    if (!canAdmin) return;
    setError("");
    try {
      const res = await apiClient.repo.actions.secrets.delete({
        owner,
        name,
        secret_name: sn,
      });
      if (!res.ok) {
        toastError(res.error.message || "Could not delete secret.");
        return;
      }
      await queryClient.invalidateQueries({ queryKey: secretsQueryKey(owner, name) });
      toastSuccess("Secret deleted.");
    } catch {
      toastError(NETWORK_ERROR);
    }
  }

  const enabled = enabledQ.data ?? true;
  const secrets = secretsQ.data ?? [];

  <section
    id="actions"
    className="mt-12 flex flex-col gap-4"
    aria-labelledby="actions-heading"
    data-testid="repo-actions-settings"
  >
    <h2
      id="actions-heading"
      className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-foreground"
    >
      Actions
    </h2>
    <p className="text-[14px] text-muted-foreground">
      Run workflows on registered self-hosted runners. Octanest does not sell managed CI minutes —
      you bring your own runners (ACT-07). Secret values are write-only after create.
    </p>

    @if (enabledQ.isPending) {
      <p className="text-[14px] text-muted-foreground">Loading Actions status…</p>;
    } @else {
      <div className="flex items-center gap-3">
        <Switch
          checked={enabled}
          disabled={!canAdmin || togglePending}
          onCheckedChange={(v) => void toggleEnabled(!!v)}
          data-testid="repo-actions-enabled"
        />
        <Label className="text-[14px]">Enable Actions for this repository</Label>
      </div>;
    }

    @if (canAdmin) {
      <div className="mt-4 space-y-3">
        <h3 className="text-[15px] font-semibold text-foreground">Repository secrets</h3>
        <form
          className="flex flex-wrap items-end gap-3"
          method="post"
          action="#"
          onSubmit={(e) => {
            e.preventDefault();
            void putSecret();
          }}
        >
          <label className="flex flex-col gap-1 text-[14px]">
            <span className="font-medium">Name</span>
            <Input
              data-testid="repo-actions-secret-name"
              value={secretName}
              onInput={(e) => setSecretName((e.target as HTMLInputElement).value)}
              autocomplete="off"
            />
          </label>
          <label className="flex flex-col gap-1 text-[14px]">
            <span className="font-medium">Value</span>
            <Input
              type="password"
              data-testid="repo-actions-secret-value"
              value={secretValue}
              onInput={(e) => setSecretValue((e.target as HTMLInputElement).value)}
              autocomplete="new-password"
            />
          </label>
          <Button
            type="submit"
            disabled={secretPending}
            data-testid="repo-actions-secret-save"
          >
            Save secret
          </Button>
        </form>
        <ul className="space-y-2">
          @for (const s of secrets; key s.name) {
            <li className="flex items-center justify-between gap-3 text-[14px]">
              <span className="font-mono">{s.name as string}</span>
              <Button
                type="button"
                variant="ghost"
                onClick={() => void deleteSecret(s.name)}
                data-testid={"repo-actions-secret-delete-" + s.name}
              >
                Delete
              </Button>
            </li>;
          } @empty {
            <li className="text-muted-foreground">No secrets yet.</li>;
          }
        </ul>
      </div>;
    }

    @if (error) {
      <p className="text-destructive" role="alert">
        {error as string}
      </p>;
    }
  </section>;
}
