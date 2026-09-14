import {
  createFileRoute,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import type { ReleasePublic, RepoPublic } from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { apiClient } from "@/lib/api-client";
import type { RepoLayoutLoaderData } from "@/lib/repo-store";
import { fetchReleaseList, fetchRepoGet } from "@/lib/ssr-repo";

export type ReleasesLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "ready"; repo: RepoPublic; releases: ReleasePublic[] };

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

async function loadReleasesPage(
  owner: string,
  repoName: string,
): Promise<ReleasesLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const listRes = await fetchReleaseList({ data: { owner, name: repoName } });
  if (!listRes.ok) {
    if (listRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: listRes.error.message || "Could not load releases.",
    };
  }
  return {
    kind: "ready",
    repo: got.data,
    releases: listRes.data.releases,
  };
}

export const Route = createFileRoute("/$owner/$repo/releases/")({
  component: RepoReleasesPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Releases · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<ReleasesLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    try {
      return await loadReleasesPage(owner, repoName);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

/** Releases list — notes-only (assets in 15-02). */
export function RepoReleasesPage() @{
  const params = useParams({ strict: false }) as {
    owner?: string;
    repo?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const layout = useLoaderData({ from: "/$owner/$repo" }) as
    | RepoLayoutLoaderData
    | undefined;
  const routeLoader = useLoaderData({ from: "/$owner/$repo/releases/" }) as
    | ReleasesLoaderData
    | undefined;

  const repoQ = useQuery({
    queryKey: ["repo", "get", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.repo.get({ owner, name: repoName });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !!owner && !!repoName,
    initialData:
      routeLoader?.kind === "ready"
        ? routeLoader.repo
        : layout?.status === "ok"
          ? (layout.repo ?? undefined)
          : undefined,
  });

  const listQ = useQuery({
    queryKey: ["release", "list", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.release.list({ owner, name: repoName });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data.releases;
    },
    enabled: !!owner && !!repoName,
    initialData:
      routeLoader?.kind === "ready" ? routeLoader.releases : undefined,
  });

  const repo = repoQ.data ?? null;
  const notFound =
    layout?.status === "not_found" ||
    routeLoader?.kind === "not_found" ||
    (repoQ.isError &&
      String(repoQ.error?.message ?? "").includes("not found"));
  const newHref = `/${owner}/${repoName}/releases/new`;
  const releases = listQ.data ?? [];

  @if (notFound || (!repoQ.isPending && !repo)) {
    <RepoNotFound />
  } @else if (!repo) {
    <p className="text-[16px] text-muted-foreground">Loading releases…</p>
  } @else {
    <>
      <RepoChrome repo={repo} active="releases" />
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-[18px] font-semibold text-foreground">Releases</h2>
        @if (repo.can_write) {
          <a href={newHref as string} className="no-underline">
            <Button type="button">Draft a new release</Button>
          </a>
        }
      </div>
      @if (listQ.isError) {
        <p role="alert" className="text-[16px] text-destructive">
          {(listQ.error?.message || NETWORK_ERROR) as string}
        </p>
      } @else if (releases.length === 0) {
        <p className="text-[16px] text-muted-foreground">
          There aren’t any releases yet.
          @if (repo.can_write) {
            <>
              {" "}
              <a href={newHref as string} className="text-primary underline">
                Create one
              </a>
              {" "}
              from an existing tag.
            </>
          }
        </p>
      } @else {
        <ul className="flex flex-col gap-4">
          @for (const rel of releases; key rel.id) {
            <li className="border-b border-border pb-4">
              <a
                href={`/${owner}/${repoName}/releases/${encodeURIComponent(rel.tag_name)}` as string}
                className="text-[16px] font-semibold text-primary no-underline hover:underline"
              >
                {(rel.title.trim() || rel.tag_name) as string}
              </a>
              <div className="mt-1 flex flex-wrap items-center gap-2 text-[14px] text-muted-foreground">
                <span>{rel.tag_name as string}</span>
                @if (rel.draft) {
                  <Badge variant="outline">Draft</Badge>
                }
                @if (rel.prerelease) {
                  <Badge variant="outline">Pre-release</Badge>
                }
                <span>
                  {(rel.author_username
                    ? `by ${rel.author_username}`
                    : "") as string}
                </span>
              </div>
            </li>
          }
        </ul>
      }
    </>
  }
}
