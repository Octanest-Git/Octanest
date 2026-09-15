import {
  createFileRoute,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import type {
  RepoPublic,
  RepoRefEntry,
  RepoTreeEntry,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { FileTree } from "@/components/repo/file-tree";
import { PathBreadcrumb } from "@/components/repo/path-breadcrumb";
import { RefSelect } from "@/components/repo/ref-select";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { parseRefAndPath, shortRefName } from "@/lib/repo-browse";
import {
  fetchRepoGet,
  fetchRepoRefs,
  fetchRepoTree,
} from "@/lib/ssr-repo";

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
    };

type Phase = { kind: "loading" } | TreeLoaderData;

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

async function loadTreePage(
  owner: string,
  repoName: string,
  splat: string | undefined,
): Promise<TreeLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const repo = got.data;
  const refsRes = await fetchRepoRefs({ data: { owner, name: repoName } });
  const refs = refsRes.ok ? refsRes.data.refs : [];
  const knownRefs = refs.map((r) => shortRefName(r.name));
  const parsed = parseRefAndPath(splat, knownRefs);
  const refName = parsed.ref || repo.default_branch || "main";
  const treeRes = await fetchRepoTree({
    data: {
      owner,
      name: repoName,
      ref: refName,
      path: parsed.path || "",
    },
  });
  if (!treeRes.ok) {
    if (treeRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: treeRes.error.message || "Could not load tree.",
    };
  }
  return {
    kind: "ready",
    repo,
    refName: shortRefName(treeRes.data.ref) || refName,
    path: treeRes.data.path || parsed.path,
    entries: treeRes.data.entries,
    refs,
  };
}

export const Route = createFileRoute("/$owner/$repo/tree/$")({
  component: RepoTreePage,
  head: ({ params }) => {
    const { path } = parseRefAndPath(
      (params as { _splat?: string })._splat,
    );
    const owner = (params as { owner: string }).owner;
    const repo = (params as { repo: string }).repo;
    const titlePath = path || "root";
    const trunc =
      titlePath.length > 40 ? `…${titlePath.slice(-36)}` : titlePath;
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

export function RepoTreePage() @{
  const params = useParams({ from: "/$owner/$repo/tree/$" }) as {
    owner: string;
    repo: string;
    _splat?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const splat = params._splat;
  const loaderData = useLoaderData({ from: "/$owner/$repo/tree/$" }) as
    | TreeLoaderData
    | undefined;

  const [phase, setPhase] = useState<Phase>(() =>
    loaderData ?? { kind: "loading" },
  );

  useEffect(() => {
    if (loaderData) {
      setPhase(loaderData);
      return;
    }
    let cancelled = false;
    (async () => {
      setPhase({ kind: "loading" });
      try {
        const next = await loadTreePage(owner, repoName, splat);
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
    <RepoNotFound />
  } @else if (phase.kind === "loading") {
    <p className="text-[16px] text-muted-foreground">Loading tree…</p>
  } @else if (phase.kind === "error") {
    <p role="alert" className="text-[16px] text-destructive">
      {phase.message as string}
    </p>
  } @else {
    <>
      <div className="mb-4 flex flex-wrap items-center gap-3">
        <RefSelect
          owner={owner}
          repo={repoName}
          refs={phase.refs}
          value={phase.refName}
          pathUnderRef={phase.path}
          mode="tree"
        />
      </div>
      <PathBreadcrumb
        owner={owner}
        repo={repoName}
        refName={phase.refName}
        path={phase.path}
      />
      <FileTree
        owner={owner}
        repo={repoName}
        refName={phase.refName}
        basePath={phase.path}
        entries={phase.entries}
      />
    </>
  }
}
