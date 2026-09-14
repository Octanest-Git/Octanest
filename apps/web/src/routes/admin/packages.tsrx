import { createFileRoute } from "@octanejs/tanstack-router";
import { useMutation, useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import {
  packagesAdminSetQuotaMutationOptions,
  packagesAdminUsageQueryOptions,
} from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";
import { Button } from "@/components/ui/button";
import { DEFAULT_OWNER_QUOTA_HINT } from "@/lib/package-quota-copy";

export const Route = createFileRoute("/admin/packages")({
  component: AdminPackagesPage,
  head: () => ({ meta: [{ title: "Packages storage · Octanest" }] }),
});

/** Admin package storage quota/usage (D-PKG-09). */
export function AdminPackagesPage() @{
  const [owner, setOwner] = useState("");
  const [lookup, setLookup] = useState("");
  const [maxBytes, setMaxBytes] = useState("");
  const qc = useQueryClient();
  const usageQ = useQuery({
    ...packagesAdminUsageQueryOptions(apiClient, { owner: lookup }),
    enabled: lookup.length > 0,
  });
  const setM = useMutation({
    ...packagesAdminSetQuotaMutationOptions(apiClient),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["packages", "adminUsage"] });
    },
  });

  @{
    <div className="mx-auto max-w-3xl px-4 py-12" data-testid="admin-packages">
      <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold">
        Package storage
      </h1>
      <p className="mt-2 text-muted-foreground">
        Usage is tracked per owner across OCI, npm, and generic formats. Override quotas below;
        defaults come from environment.
      </p>
      <dl className="mt-8 space-y-4 text-[15px]">
        <div>
          <dt className="font-medium">Max blob size</dt>
          <dd className="text-muted-foreground">OCTANEST_PACKAGES_MAX_BLOB_BYTES (default 2 GiB)</dd>
        </div>
        <div>
          <dt className="font-medium">Per-owner quota</dt>
          <dd className="text-muted-foreground">{DEFAULT_OWNER_QUOTA_HINT as string}</dd>
        </div>
      </dl>

      <form
        className="mt-10 flex flex-wrap items-end gap-3"
        onSubmit={(e) => {
          e.preventDefault();
          setLookup(owner.trim());
        }}
      >
        <label className="flex flex-col gap-1 text-[14px]">
          <span className="font-medium">Owner login</span>
          <input
            className="h-10 rounded-md border border-border bg-background px-3"
            data-testid="admin-packages-owner"
            value={owner as string}
            onInput={(e) => setOwner((e.target as HTMLInputElement).value)}
          />
        </label>
        <Button type="submit" data-testid="admin-packages-lookup">
          Load usage
        </Button>
      </form>

      @if (usageQ.isError) {
        <p className="mt-4 text-destructive" role="alert">
          {(usageQ.error as Error)?.message ?? "Failed to load usage"}
        </p>
      }
      @if (usageQ.data) {
        <div className="mt-8 space-y-4" data-testid="admin-packages-usage">
          <p className="text-[15px]">
            Used {(usageQ.data.used_bytes as number).toLocaleString()} /{" "}
            {(usageQ.data.quota_bytes as number).toLocaleString()} bytes
            {" "}(default {(usageQ.data.default_quota_bytes as number).toLocaleString()})
          </p>
          <ul className="space-y-1 text-[14px] text-muted-foreground">
            @for (const f of usageQ.data.by_format; key f.format) {
              <li>
                {(f.format as string) + ": " + (f.bytes as number).toLocaleString() + " bytes"}
              </li>
            }
          </ul>
          <ul className="mt-4 space-y-1 text-[14px]">
            @for (const p of usageQ.data.packages; key p.package_id) {
              <li data-package={p.name as string}>
                {(p.format as string) + "/" + (p.name as string) + " — " + (p.bytes as number).toLocaleString() + " B"}
              </li>
            }
          </ul>
          <form
            className="mt-6 flex flex-wrap items-end gap-3"
            onSubmit={(e) => {
              e.preventDefault();
              const n = Number(maxBytes);
              if (!lookup || !Number.isFinite(n) || n < 0) return;
              setM.mutate({ owner: lookup, max_bytes: Math.floor(n) });
            }}
          >
            <label className="flex flex-col gap-1 text-[14px]">
              <span className="font-medium">Override max bytes</span>
              <input
                className="h-10 rounded-md border border-border bg-background px-3"
                data-testid="admin-packages-max-bytes"
                value={maxBytes as string}
                onInput={(e) => setMaxBytes((e.target as HTMLInputElement).value)}
              />
            </label>
            <Button type="submit" data-testid="admin-packages-set-quota">
              Set quota
            </Button>
          </form>
        </div>
      }
    </div>
  }
}
