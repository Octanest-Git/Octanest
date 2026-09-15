import { createFileRoute } from "@octanejs/tanstack-router";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import type {
  AdminLfsSettingsPublic,
  AdminLfsUsageResponse,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { AdminAuthSkeleton } from "@/components/page-skeletons";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { apiClient } from "@/lib/api-client";
import { authSessionQueryOptions } from "@/lib/session-queries";
import { toastSuccess } from "@/lib/toast";

export const Route = createFileRoute("/admin/lfs")({
  component: AdminLfsPage,
  head: () => ({ meta: [{ title: "LFS quotas · Octanest" }] }),
});

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const FORBIDDEN = "You need admin access to manage LFS quotas.";

function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
}

/** Exported for admin/lfs integration tests (D-LFS-12 / D-LFS-13 / D-LFS-19). */
export function AdminLfsPage() @{
  const queryClient = useQueryClient();
  const me = useQuery(authSessionQueryOptions());
  const isSysAdmin = me.isSuccess && me.data?.role === "sys-admin";

  const settingsQ = useQuery({
    queryKey: ["admin", "lfs", "settings"] as const,
    enabled: isSysAdmin,
    queryFn: async (): Promise<AdminLfsSettingsPublic> => {
      const res = await apiClient.admin.lfs.getSettings();
      if (!res.ok) throw new Error(res.error.message || FORBIDDEN);
      return res.data;
    },
  });

  const usageQ = useQuery({
    queryKey: ["admin", "lfs", "usage"] as const,
    enabled: isSysAdmin,
    queryFn: async (): Promise<AdminLfsUsageResponse> => {
      const res = await apiClient.admin.lfs.getUsage();
      if (!res.ok) throw new Error(res.error.message || FORBIDDEN);
      return res.data;
    },
  });

  const [maxObject, setMaxObject] = useState("");
  const [quotaRepo, setQuotaRepo] = useState("");
  const [quotaUser, setQuotaUser] = useState("");
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (me.isPending) return;
    if (me.isError) return;
    if (me.data === null) {
      window.location.assign("/login?returnTo=/admin/lfs");
    }
  }, [me.isPending, me.isError, me.data]);

  useEffect(() => {
    if (settingsQ.isSuccess && settingsQ.data) {
      setMaxObject(String(settingsQ.data.max_object_bytes));
      setQuotaRepo(String(settingsQ.data.quota_repo_bytes));
      setQuotaUser(String(settingsQ.data.quota_user_bytes));
    }
  }, [settingsQ.isSuccess, settingsQ.dataUpdatedAt]);

  async function save() {
    setError("");
    const max_object_bytes = Number(maxObject);
    const quota_repo_bytes = Number(quotaRepo);
    const quota_user_bytes = Number(quotaUser);
    if (
      ![max_object_bytes, quota_repo_bytes, quota_user_bytes].every((n) =>
        Number.isFinite(n),
      )
    ) {
      setError("Enter valid numeric byte limits.");
      return;
    }
    setPending(true);
    try {
      const res = await apiClient.admin.lfs.updateSettings({
        max_object_bytes,
        quota_repo_bytes,
        quota_user_bytes,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not save LFS settings.");
        setPending(false);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["admin", "lfs"] });
      toastSuccess("LFS quotas updated.");
      setPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  async function clearOverrides() {
    setError("");
    setPending(true);
    try {
      const res = await apiClient.admin.lfs.updateSettings({
        clear_overrides: true,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not clear overrides.");
        setPending(false);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: ["admin", "lfs"] });
      toastSuccess("Reverted to environment defaults.");
      setPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  const loading = me.isPending;
  const signedOut = !me.isPending && (!me.isSuccess || !me.data);
  const forbidden = me.isSuccess && !!me.data && !isSysAdmin;

  @if (loading) {
    <AdminAuthSkeleton />
  } @else {
    @if (signedOut) {
      <div className="mx-auto max-w-lg py-12 text-center">
        <p className="text-[16px] text-muted-foreground">
          Sign in as a system administrator to manage LFS quotas.
        </p>
        <a
          href="/login?returnTo=/admin/lfs"
          className="mt-4 inline-block text-primary underline-offset-2 hover:underline"
        >
          Sign in
        </a>
      </div>
    } @else {
      @if (forbidden) {
        <div className="mx-auto max-w-lg py-12 text-center">
          <p role="alert" className="text-[16px] text-destructive">
            {FORBIDDEN as string}
          </p>
        </div>
      } @else {
        <div className="mx-auto max-w-3xl py-8" data-testid="admin-lfs-page">
          <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold text-foreground">
            Git LFS quotas
          </h1>
          <p className="mt-2 text-[14px] text-muted-foreground">
            Override instance defaults for max object size and storage quotas.
            Environment variables remain the fallback when overrides are cleared.
          </p>
          <p className="mt-1 text-[14px]">
            <a
              href="/admin/auth"
              className="text-primary underline-offset-2 hover:underline"
            >
              Auth settings
            </a>
          </p>

          @if (settingsQ.isPending) {
            <p className="mt-8 text-[14px] text-muted-foreground">Loading…</p>
          } @else {
            @if (settingsQ.isError) {
              <p role="alert" className="mt-8 text-[14px] text-destructive">
                {(settingsQ.error instanceof Error
                  ? settingsQ.error.message
                  : FORBIDDEN) as string}
              </p>
            } @else {
              <form
                method="post"
                action="#"
                className="mt-8 flex flex-col gap-4"
                onSubmit={(e) => {
                  e.preventDefault();
                  void save();
                }}
              >
                <div className="flex flex-col gap-2">
                  <Label htmlFor="lfs-max-object">Max object bytes</Label>
                  <Input
                    id="lfs-max-object"
                    inputMode="numeric"
                    value={maxObject}
                    onInput={(e) =>
                      setMaxObject((e.currentTarget as HTMLInputElement).value)
                    }
                    disabled={pending}
                  />
                  @if (settingsQ.data?.max_object_bytes_overridden) {
                    <span className="text-[13px] text-muted-foreground">
                      Override active
                    </span>
                  }
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="lfs-quota-repo">Per-repo quota bytes</Label>
                  <Input
                    id="lfs-quota-repo"
                    inputMode="numeric"
                    value={quotaRepo}
                    onInput={(e) =>
                      setQuotaRepo((e.currentTarget as HTMLInputElement).value)
                    }
                    disabled={pending}
                  />
                  @if (settingsQ.data?.quota_repo_bytes_overridden) {
                    <span className="text-[13px] text-muted-foreground">
                      Override active
                    </span>
                  }
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="lfs-quota-user">Per-user quota bytes</Label>
                  <Input
                    id="lfs-quota-user"
                    inputMode="numeric"
                    value={quotaUser}
                    onInput={(e) =>
                      setQuotaUser((e.currentTarget as HTMLInputElement).value)
                    }
                    disabled={pending}
                  />
                  @if (settingsQ.data?.quota_user_bytes_overridden) {
                    <span className="text-[13px] text-muted-foreground">
                      Override active
                    </span>
                  }
                </div>
                @if (error) {
                  <p role="alert" className="text-[14px] text-destructive">
                    {error as string}
                  </p>
                }
                <div className="flex flex-wrap gap-2">
                  <Button type="submit" disabled={pending}>
                    {(pending ? "Saving…" : "Save quotas") as string}
                  </Button>
                  <Button
                    type="button"
                    variant="secondary"
                    disabled={pending}
                    onClick={() => void clearOverrides()}
                  >
                    Clear overrides
                  </Button>
                </div>
              </form>
            }
          }

          <section
            className="mt-12 flex flex-col gap-4"
            aria-labelledby="lfs-inst-usage"
          >
            <h2
              id="lfs-inst-usage"
              className="font-[family-name:var(--font-display)] text-[18px] font-semibold"
            >
              Instance usage breakdown
            </h2>
            @if (usageQ.isPending) {
              <p className="text-[14px] text-muted-foreground">Loading usage…</p>
            } @else {
              @if (usageQ.isError) {
                <p role="alert" className="text-[14px] text-destructive">
                  {(usageQ.error instanceof Error
                    ? usageQ.error.message
                    : "Could not load usage.") as string}
                </p>
              } @else {
                @if (usageQ.data) {
                  <>
                    <p className="text-[14px] text-muted-foreground">
                      {(
                        `${usageQ.data.object_count} unique OIDs · ${formatBytes(usageQ.data.physical_bytes)} physical · ${formatBytes(usageQ.data.logical_bytes)} logical`
                      ) as string}
                    </p>
                    <div className="grid gap-6 sm:grid-cols-2">
                      <div>
                        <h3 className="text-[15px] font-medium">By repository</h3>
                        @if (usageQ.data.by_repo.length === 0) {
                          <p className="mt-2 text-[13px] text-muted-foreground">
                            No linked LFS objects.
                          </p>
                        } @else {
                          <ul className="mt-2 flex flex-col gap-1 text-[13px] text-muted-foreground">
                            @for (const row of usageQ.data.by_repo; key row.repository_id) {
                              <li>
                                {(
                                  `${row.owner}/${row.name} · ${row.object_count} · ${formatBytes(row.logical_bytes)}`
                                ) as string}
                              </li>
                            }
                          </ul>
                        }
                      </div>
                      <div>
                        <h3 className="text-[15px] font-medium">By owner</h3>
                        @if (usageQ.data.by_owner.length === 0) {
                          <p className="mt-2 text-[13px] text-muted-foreground">
                            No linked LFS objects.
                          </p>
                        } @else {
                          <ul className="mt-2 flex flex-col gap-1 text-[13px] text-muted-foreground">
                            @for (const row of usageQ.data.by_owner; key row.owner_id) {
                              <li>
                                {(
                                  `${row.owner_slug} · ${row.object_count} · ${formatBytes(row.logical_bytes)}`
                                ) as string}
                              </li>
                            }
                          </ul>
                        }
                      </div>
                    </div>
                  </>
                }
              }
            }
          </section>
        </div>
      }
    }
  }
}
