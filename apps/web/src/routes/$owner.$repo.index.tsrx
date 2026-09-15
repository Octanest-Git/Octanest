import { createFileRoute, useLoaderData, useParams } from "@octanejs/tanstack-router";
import type { RepoPublic, RepoRefEntry, RepoTreeEntry } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { FileTree } from "@/components/repo/file-tree";
import { CloneBox } from "@/components/repo/clone-box";
import { QuickSetup } from "@/components/repo/quick-setup";
import { ReadmePanel } from "@/components/repo/readme-panel";
import { RepoBrowseToolbar } from "@/components/repo/repo-browse-toolbar";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { findReadmeName, shortRefName } from "@/lib/repo-browse";
import { renderGfm } from "@/lib/markdown";
import { fetchRepoBlob, fetchRepoGet, fetchRepoRefs, fetchRepoTree } from "@/lib/ssr-repo";

export type CodeHomeLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "empty"; repo: RepoPublic }
  | {
      kind: "ready";
      repo: RepoPublic;
      refName: string;
      entries: RepoTreeEntry[];
      refs: RepoRefEntry[];
      /** Pre-rendered sanitized HTML (no client markdown skeleton). */
      readme: { name: string; html: string } | null;
    };

async function loadCodeHome(
  owner: string,
  repoName: string,
  fetchGet: typeof fetchRepoGet,
  fetchTree: typeof fetchRepoTree,
  fetchRefs: typeof fetchRepoRefs,
  fetchBlob: typeof fetchRepoBlob,
): Promise<CodeHomeLoaderData> {
  const got = await fetchGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const repo = got.data;
  const refName = repo.default_branch || "main";
  const [treeRes, refsRes] = await Promise.all([
    fetchTree({ data: { owner, name: repoName, ref: refName, path: "" } }),
    fetchRefs({ data: { owner, name: repoName } }),
  ]);
  if (!treeRes.ok) {
    if (treeRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: treeRes.error.message || "Could not load tree.",
    };
  }
  const refs = refsRes.ok ? refsRes.data.refs : [];
  if (treeRes.data.empty) {
    return { kind: "empty", repo };
  }
  const entries = treeRes.data.entries;
  const readmeName = findReadmeName(entries);
  let readme: { name: string; html: string } | null = null;
  if (readmeName) {
    const blobRes = await fetchBlob({
      data: { owner, name: repoName, ref: refName, path: readmeName },
    });
    if (
      blobRes.ok &&
      !blobRes.data.is_binary &&
      blobRes.data.encoding === "utf-8" &&
      blobRes.data.content
    ) {
      readme = {
        name: readmeName,
        html: await renderGfm(blobRes.data.content),
      };
    }
  }
  return {
    kind: "ready",
    repo,
    refName: shortRefName(treeRes.data.ref) || refName,
    entries,
    refs,
    readme,
  };
}

export const Route = createFileRoute("/$owner/$repo/")({
  component: RepoCodeHome,
  head: ({ params }) => ({
    meta: [
      {
        title: `${params.owner}/${params.repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<CodeHomeLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    try {
      return await loadCodeHome(
        owner,
        repoName,
        fetchRepoGet,
        fetchRepoTree,
        fetchRepoRefs,
        fetchRepoBlob,
      );
    } catch {
      return {
        kind: "error",
        message: "Can't reach Octanest. Check your connection and try again.",
      };
    }
  },
});

type BrowsePhase = { kind: "loading" } | CodeHomeLoaderData;

/** Code home — Quick setup when empty; tree + README when populated (D-15). */
export function RepoCodeHome() @{
  const params = useParams({ from: "/$owner/$repo/" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const loaderData = useLoaderData({ from: "/$owner/$repo/" }) as CodeHomeLoaderData | undefined;

  const [phase, setPhase] = useState<BrowsePhase>(() =>
    loaderData ? loaderData : { kind: "loading" },
  );

  // Client fallback for unit tests / soft navigations without loader payload.
  useEffect(() => {
    if (loaderData) {
      setPhase(loaderData);
      return;
    }
    let cancelled = false;
    void (async () => {
      setPhase({ kind: "loading" });
      try {
        const next = await loadCodeHome(
          owner,
          repoName,
          async ({ data }) => apiClient.repo.get(data) as never,
          async ({ data }) => apiClient.repo.tree(data) as never,
          async ({ data }) => apiClient.repo.refs(data) as never,
          async ({ data }) => apiClient.repo.blob(data) as never,
        );
        if (!cancelled) setPhase(next);
      } catch {
        if (!cancelled) {
          setPhase({
            kind: "error",
            message: "Can't reach Octanest. Check your connection and try again.",
          });
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, loaderData]);

  // Chrome comes from `$owner.$repo` layout (D-QH-01).
  @if (phase.kind === "not_found") {
    <RepoNotFound />;
  } @else {
    <>
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-48 w-full" />
        </div>;
      } @else if (phase.kind === "error") {
        <p role="alert" className="text-[16px] text-destructive">
          {phase.message as string}
        </p>;
      } @else if (phase.kind === "empty") {
        <>
          <div className="mb-4 flex flex-wrap items-center justify-end gap-3">
            <CloneBox
              owner={owner}
              repo={repoName}
              refName={phase.repo.default_branch || "main"}
              empty={true}
            />
          </div>
          <QuickSetup owner={owner} repo={repoName} />
        </>;
      } @else {
        <>
          <RepoBrowseToolbar
            owner={owner}
            repo={repoName}
            refName={phase.refName}
            refs={phase.refs}
            empty={false}
          />
          <FileTree
            owner={owner}
            repo={repoName}
            refName={phase.refName}
            basePath=""
            entries={phase.entries}
          />
          @if (phase.readme) {
            <ReadmePanel html={phase.readme.html} fileName={phase.readme.name} />;
          }
        </>;
      }
    </>;
  }
}
