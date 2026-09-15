import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import { packagesListQueryOptions } from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";
import { useRepoStore } from "@/lib/repo-store";

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

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

/**
 * Repo-linked packages (D-PKG-11) — scoped by repository_id.
 * Chrome/shell come from `$owner.$repo` layout (D-QH-01).
 */
export function RepoPackagesPage() @{
  const params = useParams({ from: "/$owner/$repo/packages" }) as {
    owner?: string;
    repo?: string;
  };
  const owner = String(params.owner ?? "");
  const repoName = String(params.repo ?? "");

  const layoutRepo = useRepoStore((s) => s.repo);
  const repositoryId = layoutRepo?.id ?? null;

  // Fallback repo.get when rendered outside layout (tests / edge); still scopes by repository_id.
  const repoQ = useQuery({
    queryKey: ["repo", "get", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.repo.get({ owner, name: repoName });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !repositoryId && !!owner && !!repoName,
  });

  const scopedRepositoryId = repositoryId ?? repoQ.data?.id ?? null;
  const listQ = useQuery({
    ...packagesListQueryOptions(apiClient, {
      repository_id: scopedRepositoryId,
    }),
    enabled: !!scopedRepositoryId,
  });

  const packages = listQ.data?.packages ?? [];
  const loading = (!repositoryId && repoQ.isPending) || (!!scopedRepositoryId && listQ.isPending);

  @{
    <div data-testid="repo-packages">
      <h2 className="font-[family-name:var(--font-display)] text-[20px] font-semibold text-foreground">
        Packages
      </h2>
      <p className="mt-1 text-muted-foreground">Packages linked to this repository.</p>
      @if (loading) {
        <p className="mt-8 text-muted-foreground">Loading packages…</p>;
      } @else {
        <ul className="mt-8 space-y-3">
          @for (const pkg of packages; key pkg.id) {
            <li className="flex items-center gap-2">
              <span>{pkg.name as string}</span>
              <span className="text-[12px] uppercase text-muted-foreground">
                {pkg.format as string}
              </span>
            </li>;
          } @empty {
            <li className="text-muted-foreground">No linked packages.</li>;
          }
        </ul>;
      }
    </div>;
    };
}
