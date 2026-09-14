import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { packagesListQueryOptions } from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";

export const Route = createFileRoute("/$owner/$repo/packages")({
  component: RepoPackagesPage,
  head: ({ params }) => {
    const p = params as { owner?: string; repo?: string };
    return {
      meta: [
        {
          title: `${String(p.owner ?? "")}/${String(p.repo ?? "")} packages · Octanest`,
        },
      ],
    };
  },
});

/** Repo-linked packages (D-PKG-11). */
export function RepoPackagesPage() @{
  const params = useParams({ from: "/$owner/$repo/packages" }) as {
    owner?: string;
    repo?: string;
  };
  const owner = String(params.owner ?? "");
  const repo = String(params.repo ?? "");
  const listQ = useQuery(packagesListQueryOptions(apiClient, { owner }));
  const packages = (listQ.data?.packages ?? []).filter(
    (p) => p.repository_id != null && p.repository_id.length > 0,
  );

  @{
    <div className="mx-auto max-w-6xl px-4 py-12" data-testid="repo-packages">
      <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold">
        {(owner + "/" + repo + " packages") as string}
      </h1>
      <p className="mt-1 text-muted-foreground">Packages linked to this repository.</p>
      <ul className="mt-8 space-y-3">
        @for (const pkg of packages; key pkg.id) {
          <li className="flex items-center gap-2">
            <span>{pkg.name as string}</span>
            <span className="text-[12px] uppercase text-muted-foreground">
              {pkg.format as string}
            </span>
          </li>
        } @empty {
          <li className="text-muted-foreground">No linked packages.</li>
        }
      </ul>
    </div>
  }
}
