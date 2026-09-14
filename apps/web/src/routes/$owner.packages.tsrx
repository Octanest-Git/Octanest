import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useMutation, useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import {
  packagesDeleteVersionMutationOptions,
  packagesListQueryOptions,
  type PackagePublic,
} from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";
import { Button } from "@/components/ui/button";
import { DeleteVersionDialog } from "@/components/packages/delete-version-dialog";

export const Route = createFileRoute("/$owner/packages")({
  component: OwnerPackagesPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `${String((params as { owner?: string }).owner ?? "")} packages · Octanest`,
      },
    ],
  }),
});

/** Owner packages list (D-PKG-11) — formats oci/npm/generic. */
export function OwnerPackagesPage() @{
  const params = useParams({ from: "/$owner/packages" }) as { owner?: string };
  const owner = String(params.owner ?? "");
  const qc = useQueryClient();
  const listQ = useQuery(packagesListQueryOptions(apiClient, { owner }));
  const [pendingDelete, setPendingDelete] = useState<{
    pkg: PackagePublic;
    version: string;
  } | null>(null);
  const delM = useMutation({
    ...packagesDeleteVersionMutationOptions(apiClient),
    onSuccess: async () => {
      setPendingDelete(null);
      await qc.invalidateQueries({ queryKey: ["packages", "list"] });
    },
  });

  const packages = listQ.data?.packages ?? [];

  @{
    <div className="mx-auto max-w-6xl px-4 py-12" data-testid="owner-packages">
      <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold text-foreground">
        Packages
      </h1>
      <p className="mt-1 text-[16px] text-muted-foreground">
        {"Registry packages for @" + (owner as string)}
      </p>
      @if (listQ.isPending) {
        <p className="mt-8 text-muted-foreground">Loading packages…</p>
      } @else {
        <ul className="mt-8 space-y-4">
          @for (const pkg of packages; key pkg.id) {
            <li className="border-b border-border pb-4">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-medium text-foreground">{pkg.name as string}</span>
                <span
                  className="rounded border border-border px-2 py-0.5 text-[12px] uppercase text-muted-foreground"
                  data-format={pkg.format as string}
                >
                  {pkg.format as string}
                </span>
              </div>
              <ul className="mt-2 space-y-1 text-[14px]">
                @for (const v of pkg.versions; key v.version) {
                  <li className="flex items-center gap-3">
                    <a
                      className="text-foreground underline-offset-4 hover:underline"
                      href={`/${owner}/packages#${pkg.name}@${v.version}`}
                    >
                      {(pkg.name + "@" + v.version) as string}
                    </a>
                    <Button
                      type="button"
                      variant="ghost"
                      className="h-8 text-[13px]"
                      onClick={() => setPendingDelete({ pkg, version: v.version })}
                    >
                      Delete
                    </Button>
                  </li>
                }
              </ul>
            </li>
          } @empty {
            <li className="text-muted-foreground">No packages yet.</li>
          }
        </ul>
      }
      @if (pendingDelete) {
        <DeleteVersionDialog
          confirmLabel={(pendingDelete.pkg.name + "@" + pendingDelete.version) as string}
          onCancel={() => setPendingDelete(null)}
          onConfirm={(confirm) => {
            delM.mutate({
              package_id: pendingDelete.pkg.id,
              version: pendingDelete.version,
              confirm,
            });
          }}
        />
      }
    </div>
  }
}
