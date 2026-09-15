import {
  Outlet,
  createFileRoute,
  useLoaderData,
  useLocation,
  useParams,
} from "@octanejs/tanstack-router";
import type { UserPublic } from "@octanest/api-client";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { RepoPageShell } from "@/components/repo/repo-page-shell";
import { RepoStoreProvider } from "@/components/repo/repo-store-provider";
import { Skeleton } from "@/components/ui/skeleton";
import { repoChromeActiveFromPath } from "@/lib/repo-chrome-active";
import { type RepoLayoutLoaderData, useRepoStore } from "@/lib/repo-store";
import { fetchSessionMe } from "@/lib/ssr-auth";
import { fetchPublicOrigin, fetchRepoGet } from "@/lib/ssr-repo";

export type { RepoLayoutLoaderData };

/**
 * Layout for `/{owner}/{repo}` — fixed max-w-6xl shell + SSR cookie-forward
 * repo/session/public origin so private repos and clone URLs paint without CLS.
 */
export const Route = createFileRoute("/$owner/$repo")({
  component: RepoLayout,
  loader: async ({ params }): Promise<RepoLayoutLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const publicOrigin = await fetchPublicOrigin();

    let me: UserPublic | null = null;
    try {
      const meRes = await fetchSessionMe();
      if (meRes.ok) me = meRes.data;
    } catch {
      /* anonymous */
    }

    try {
      const got = await fetchRepoGet({ data: { owner, name: repoName } });
      if (!got.ok) {
        if (got.error.code === "repo.not_found") {
          return {
            owner,
            repoName,
            status: "not_found",
            repo: null,
            me,
            message: "",
            publicOrigin,
          };
        }
        return {
          owner,
          repoName,
          status: "error",
          repo: null,
          me,
          message: got.error.message || "Could not load repository.",
          publicOrigin,
        };
      }
      return {
        owner,
        repoName,
        status: "ok",
        repo: got.data,
        me,
        message: "",
        publicOrigin,
      };
    } catch {
      return {
        owner,
        repoName,
        status: "error",
        repo: null,
        me,
        message: "Can't reach Octanest. Check your connection and try again.",
        publicOrigin,
      };
    }
  },
});

function RepoLayout() @{
  const params = useParams({ from: "/$owner/$repo" }) as {
    owner: string;
    repo: string;
  };
  const data = useLoaderData({ from: "/$owner/$repo" }) as RepoLayoutLoaderData | undefined;

  const owner = data?.owner ?? params.owner ?? "";
  const repoName = data?.repoName ?? params.repo ?? "";
  const status = data?.status ?? "ok";
  const repo = data?.repo ?? null;
  const me = data?.me ?? null;
  const message = data?.message ?? "";
  const publicOrigin = data?.publicOrigin ?? "";

  @if (!data) {
    <RepoPageShell>
      <div className="flex flex-col gap-4" aria-busy="true">
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-48 w-full" />
      </div>
    </RepoPageShell>;
  } @else if (status === "not_found") {
    <RepoNotFound />;
  } @else if (status === "error") {
    <RepoPageShell>
      <p role="alert" className="text-[16px] text-destructive">
        {message as string}
      </p>
    </RepoPageShell>;
  } @else {
    <RepoStoreProvider
      key={`${owner}/${repoName}` as string}
      owner={owner}
      repoName={repoName}
      status={status}
      repo={repo}
      me={me}
      message={message}
      publicOrigin={publicOrigin}
    >
      <RepoPageShell>
        <RepoLayoutChrome />
      </RepoPageShell>
    </RepoStoreProvider>;
  }
}

/** Shared repo HI chrome above leaf Outlet (D-QH-01). Leaves may still remount until 11.1-03. */
function RepoLayoutChrome() @{
  const repo = useRepoStore((s) => s.repo);
  const pathname = useLocation({
    select: (loc) => (loc as { pathname?: string }).pathname,
  });
  const active = repoChromeActiveFromPath(String(pathname ?? ""));

  @if (repo) {
    <>
      <RepoChrome repo={repo} active={active} />
      <Outlet />
    </>;
  } @else {
    <Outlet />;
  }
}
