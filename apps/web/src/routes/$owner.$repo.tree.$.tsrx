import { createFileRoute, useLoaderData, useParams } from "@octanejs/tanstack-router";
import type { RepoPublic, RepoRefEntry, RepoTreeEntry } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { FileTree } from "@/components/repo/file-tree";
import { RepoBrowseToolbar } from "@/components/repo/repo-browse-toolbar";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { ReadmePanel } from "@/components/repo/readme-panel";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { findReadmeName, joinRepoPath, parseRefAndPath, shortRefName } from "@/lib/repo-browse";
import { renderGfm } from "@/lib/markdown";
import { fetchRepoBlob, fetchRepoGet, fetchRepoRefs, fetchRepoTree } from "@/lib/ssr-repo";

export type TreeLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      refName: string;
      path: string;
      entries: RepoTreeEntry[];
      refs: RepoRefEntry[];
      /** Nested-dir README when present (same rules as code home). */
      readme: { name: string; html: string } | null;
    };

type Phase = { kind: "loading" } | TreeLoaderData;

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

async function loadTreePage(
  owner: string,
  repoName: string,
  splat: string | undefined,
  fetchGet: typeof fetchRepoGet = fetchRepoGet,
  fetchTree: typeof fetchRepoTree = fetchRepoTree,
  fetchRefs: typeof fetchRepoRefs = fetchRepoRefs,
  fetchBlob: typeof fetchRepoBlob = fetchRepoBlob,
): Promise<TreeLoaderData> {
  const got = await fetchGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const repo = got.data;
  const refsRes = await fetchRefs({ data: { owner, name: repoName } });
  const refs = refsRes.ok ? refsRes.data.refs : [];
  const knownRefs = refs.map((r) => shortRefName(r.name));
  const parsed = parseRefAndPath(splat, knownRefs);
  const refName = parsed.ref || repo.default_branch || "main";
  const path = parsed.path || "";
  const treeRes = await fetchTree({
    data: {
      owner,
      name: repoName,
      ref: refName,
      path,
    },
  });
  if (!treeRes.ok) {
    if (treeRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: treeRes.error.message || "Could not load tree.",
    };
  }
  const entries = treeRes.data.entries;
  const dirPath = treeRes.data.path || path;
  const readmeName = findReadmeName(entries);
  let readme: { name: string; html: string } | null = null;
  if (readmeName) {
    const blobRes = await fetchBlob({
      data: {
        owner,
        name: repoName,
        ref: refName,
        path: joinRepoPath(dirPath, readmeName),
      },
    });
    if (
      blobRes.ok &&
      !blobRes.data.is_binary &&
      blobRes.data.encoding === "utf-8" &&
      blobRes.data.content
    ) {
      readme = {
        name: readmeName,
        html: await renderGfm(blobRes.data.content, {
          owner,
          repo: repoName,
        }),
      };
    }
  }
  return {
    kind: "ready",
    repo,
    refName: shortRefName(treeRes.data.ref) || refName,
    path: dirPath,
    entries,
    refs,
    readme,
  };
}

export const Route = createFileRoute("/$owner/$repo/tree/$")({
  component: RepoTreePage,
  head: ({ params }) => {
    const { path } = parseRefAndPath((params as { _splat?: string })._splat);
    const owner = (params as { owner: string }).owner;
    const repo = (params as { repo: string }).repo;
    const titlePath = path || "root";
    const trunc = titlePath.length > 40 ? `…${titlePath.slice(-36)}` : titlePath;
    return {
      meta: [{ title: `${trunc} · ${owner}/${repo} · Octanest` }],
    };
  },
  loader: async ({ params }): Promise<TreeLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const splat = (params as { _splat?: string })._splat;
    try {
      return await loadTreePage(owner, repoName, splat);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

/** Nested tree — same toolbar / file list / README chrome as code home (GitHub/Gitea). */
export function RepoTreePage() @{
  const params = useParams({ from: "/$owner/$repo/tree/$" }) as {
    owner: string;
    repo: string;
    _splat?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const splat = params._splat;
  const loaderData = useLoaderData({ from: "/$owner/$repo/tree/$" }) as TreeLoaderData | undefined;

  const [phase, setPhase] = useState<Phase>(() => loaderData ?? { kind: "loading" });

  useEffect(() => {
    if (loaderData) {
      setPhase(loaderData);
      return;
    }
    let cancelled = false;
    void (async () => {
      setPhase({ kind: "loading" });
      try {
        const next = await loadTreePage(
          owner,
          repoName,
          splat,
          async ({ data }) => apiClient.repo.get(data) as never,
          async ({ data }) => apiClient.repo.tree(data) as never,
          async ({ data }) => apiClient.repo.refs(data) as never,
          async ({ data }) => apiClient.repo.blob(data) as never,
        );
        if (!cancelled) setPhase(next);
      } catch {
        if (!cancelled) setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, splat, loaderData]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />;
  } @else if (phase.kind === "loading") {
    <div className="flex flex-col gap-4" aria-busy="true" data-testid="repo-tree-loading">
      <Skeleton className="h-8 w-64" />
      <Skeleton className="h-10 w-full" />
      <Skeleton className="h-48 w-full" />
    </div>;
  } @else if (phase.kind === "error") {
    <p role="alert" className="text-[16px] text-destructive">
      {phase.message as string}
    </p>;
  } @else {
    <>
      <RepoBrowseToolbar
        owner={owner}
        repo={repoName}
        refName={phase.refName}
        refs={phase.refs}
        path={phase.path}
        showPathCrumbs={true}
        empty={false}
      />
      <FileTree
        owner={owner}
        repo={repoName}
        refName={phase.refName}
        basePath={phase.path}
        entries={phase.entries}
      />
      @if (phase.readme) {
        <ReadmePanel html={phase.readme.html} fileName={phase.readme.name} />;
      }
    </>;
  }
}
