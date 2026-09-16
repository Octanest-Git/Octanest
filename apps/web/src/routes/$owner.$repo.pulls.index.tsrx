import { createFileRoute, useLoaderData, useParams } from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import type { PullListResponse, RepoPublic } from "@octanest/api-client";
import { useState } from "octane";
import { PullsList } from "@/components/repo/pulls-list";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { apiClient } from "@/lib/api-client";
import { fetchRepoGet } from "@/lib/ssr-repo";

export type PullsLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      list: PullListResponse;
      state: "open" | "closed" | "all";
    };

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const PAGE_SIZE = 25;

async function loadPullsPage(
  owner: string,
  repoName: string,
  state: "open" | "closed" | "all",
): Promise<PullsLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const listRes = await apiClient.pull.list({
    owner,
    name: repoName,
    state,
    offset: 0,
    limit: PAGE_SIZE,
  });
  if (!listRes.ok) {
    if (listRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: listRes.error.message || "Could not load pull requests.",
    };
  }
  return {
    kind: "ready",
    repo: got.data,
    list: listRes.data,
    state,
  };
}

export const Route = createFileRoute("/$owner/$repo/pulls/")({
  component: PullsListPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Pulls · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<PullsLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    try {
      return await loadPullsPage(owner, repoName, "open");
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

export function PullsListPage() @{
  const params = useParams({ strict: false }) as {
    owner?: string;
    repo?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const routeLoader = useLoaderData({ from: "/$owner/$repo/pulls/" }) as
    | PullsLoaderData
    | undefined;

  const [state, setState] = useState<"open" | "closed" | "all">("open");

  const listQ = useQuery({
    queryKey: ["pull", "list", owner, repoName, state] as const,
    queryFn: async () => {
      const res = await apiClient.pull.list({
        owner,
        name: repoName,
        state,
        offset: 0,
        limit: PAGE_SIZE,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !!owner && !!repoName,
    initialData: routeLoader?.kind === "ready" && state === "open" ? routeLoader.list : undefined,
  });

  const repo = routeLoader?.kind === "ready" ? routeLoader.repo : null;
  const list = listQ.data;

  @if (routeLoader?.kind === "not_found") {
    <RepoNotFound />;
  } @else {
    @if (routeLoader?.kind === "error") {
      <p className="text-destructive">{routeLoader.message as string}</p>;
    } @else {
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex gap-2" role="tablist" aria-label="Pull state">
            <button
              type="button"
              className={
                (state === "open"
                  ? "rounded-md bg-muted px-3 py-1.5 text-sm font-medium"
                  : "rounded-md px-3 py-1.5 text-sm text-muted-foreground") as string
              }
              onClick={() => setState("open")}
            >
              Open
            </button>
            <button
              type="button"
              className={
                (state === "closed"
                  ? "rounded-md bg-muted px-3 py-1.5 text-sm font-medium"
                  : "rounded-md px-3 py-1.5 text-sm text-muted-foreground") as string
              }
              onClick={() => setState("closed")}
            >
              Closed
            </button>
            <button
              type="button"
              className={
                (state === "all"
                  ? "rounded-md bg-muted px-3 py-1.5 text-sm font-medium"
                  : "rounded-md px-3 py-1.5 text-sm text-muted-foreground") as string
              }
              onClick={() => setState("all")}
            >
              All
            </button>
          </div>
          @if (repo?.can_write) {
            <a
              href={`/${owner}/${repoName}/pulls/new` as string}
              className="inline-flex items-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground no-underline"
            >
              New pull request
            </a>;
          }
        </div>
        @if (list) {
          <PullsList
            owner={owner as string}
            repo={repoName as string}
            list={list as PullListResponse}
          />;
        } @else {
          <p className="text-muted-foreground">Loading pull requests…</p>;
        }
      </div>;
    }
  }
}
