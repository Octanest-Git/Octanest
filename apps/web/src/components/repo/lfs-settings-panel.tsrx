import type { RepoLfsObjectEntry, RepoLfsUsageResponse } from "@octanest/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { apiClient } from "@/lib/api-client";
import { toastSuccess } from "@/lib/toast";

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

const DOCS_HREF = "/docs/CONFIGURATION.md#git-lfs";

function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
}

function lfsUsageQueryKey(owner: string, name: string) {
  return ["repo", "lfs", "usage", owner, name] as const;
}

/** Repo Settings LFS enable + usage (D-LFS-10 / D-LFS-16 / D-LFS-17 / D-LFS-19). */
export function LfsSettingsPanel(props: {
  owner: string;
  name: string;
  /** Server Capability Admin — toggle only when true (D-LFS-10). */
  can_admin: boolean;
}) @{
  const owner = props.owner;
  const name = props.name;
  const canAdmin = props.can_admin;
  const queryClient = useQueryClient();

  const usageQ = useQuery({
    queryKey: lfsUsageQueryKey(owner, name),
    queryFn: async (): Promise<RepoLfsUsageResponse> => {
      const res = await apiClient.repo.lfs.getUsage({ owner, name });
      if (!res.ok) throw new Error(res.error.message || "Could not load LFS usage.");
      return res.data;
    },
  });

  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");

  async function toggleEnabled(next: boolean) {
    if (!canAdmin) return;
    setError("");
    setPending(true);
    try {
      const res = await apiClient.repo.lfs.setEnabled({
        owner,
        name,
        enabled: next,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not update LFS setting.");
        setPending(false);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: lfsUsageQueryKey(owner, name) });
      toastSuccess(next ? "Git LFS enabled." : "Git LFS disabled.");
      setPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  const usage = usageQ.data;
  const enabled = usage?.enabled ?? false;
  const objects: RepoLfsObjectEntry[] = usage?.objects ?? [];

  <section
    className="mt-12 flex flex-col gap-4"
    aria-labelledby="lfs-heading"
    data-testid="repo-lfs-settings"
  >
    <h2
      id="lfs-heading"
      className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-foreground"
    >
      Git LFS
    </h2>
    <p className="text-[14px] text-muted-foreground">
      Enable Git LFS for this repository. Track large files with{" "}
      <code className="text-[13px]">git lfs track</code> and commit{" "}
      <code className="text-[13px]">.gitattributes</code> yourself — Octanest does not auto-commit
      attributes.{" "}
      <a href={DOCS_HREF as string} className="text-primary underline-offset-2 hover:underline">
        LFS setup docs
      </a>
    </p>

    @if (usageQ.isPending) {
      <p className="text-[14px] text-muted-foreground">Loading LFS status…</p>;
    } @else if (usageQ.isError) {
      <p role="alert" className="text-[14px] text-destructive">
        {
          (usageQ.error instanceof Error
            ? usageQ.error.message
            : "Could not load LFS status.") as string
        }
      </p>;
    } @else {
      <>
        <p className="text-[14px] text-foreground">
          Status: <strong>{(enabled ? "Enabled" : "Disabled") as string}</strong>
        </p>

        @if (canAdmin) {
          <div className="flex items-center gap-3">
            <Switch
              id="lfs-enable"
              checked={enabled}
              disabled={pending}
              onCheckedChange={(v) => void toggleEnabled(!!v)}
              aria-label="Enable Git LFS for this repository"
            />
            <Label htmlFor="lfs-enable" className="text-[14px]">
              Enable Git LFS for this repository
            </Label>
          </div>;
        } @else {
          <p className="text-[14px] text-muted-foreground">
            Only repository admins can enable or disable LFS.
          </p>;
        }

        @if (error) {
          <p role="alert" className="text-[14px] text-destructive">
            {error as string}
          </p>;
        }

        <div
          className="flex flex-col gap-2 rounded-md border border-border p-4"
          aria-labelledby="lfs-usage-heading"
        >
          <h3 id="lfs-usage-heading" className="text-[15px] font-medium text-foreground">
            Usage breakdown
          </h3>
          <p className="text-[14px] text-muted-foreground">
            {
              (usage
                ? `${usage.object_count} object${usage.object_count === 1 ? "" : "s"} · ${formatBytes(usage.logical_bytes)} logical · quota ${formatBytes(usage.quota_repo_bytes)}`
                : "No usage data") as string
            }
          </p>
          @if (objects.length > 0) {
            <ul className="mt-2 flex flex-col gap-1 text-[13px] font-mono text-muted-foreground">
              @for (const obj of objects; key obj.oid) {
                <li>
                  {
                    (obj.oid.slice(0, 12) +
                      "… · " +
                      formatBytes(obj.size)) as string
                  }
                </li>;
              }
            </ul>;
          } @else {
            <p className="text-[14px] text-muted-foreground">
              No LFS objects linked yet.
            </p>;
          }
        </div>
      </>;
    }
  </section>;
}
