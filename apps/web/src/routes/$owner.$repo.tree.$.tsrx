import {
  createFileRoute,
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
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { parseRefAndPath, shortRefName } from "@/lib/repo-browse";

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
});

type Phase =
  | { kind: "loading" }
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

export function RepoTreePage() @{
  const params = useParams({ from: "/$owner/$repo/tree/$" }) as {
    owner: string;
    repo: string;
    _splat?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const parsed = parseRefAndPath(params._splat);
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setPhase({ kind: "loading" });
      try {
        const got = await apiClient.repo.get({ owner, name: repoName });
        if (cancelled) return;
        if (!got.ok) {
          setPhase(
            got.error.code === "repo.not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: got.error.message || "Could not load repository.",
                },
          );
          return;
        }
        const repo = got.data;
        const refName = parsed.ref || repo.default_branch || "main";
        const [treeRes, refsRes] = await Promise.all([
          apiClient.repo.tree({
            owner,
            name: repoName,
            ref: refName,
            path: parsed.path || "",
          }),
          apiClient.repo.refs({ owner, name: repoName }),
        ]);
        if (cancelled) return;
        if (!treeRes.ok) {
          setPhase(
            treeRes.error.code === "repo.not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: treeRes.error.message || "Could not load tree.",
                },
          );
          return;
        }
        setPhase({
          kind: "ready",
          repo,
          refName: shortRefName(treeRes.data.ref) || refName,
          path: treeRes.data.path || parsed.path,
          entries: treeRes.data.entries,
          refs: refsRes.ok ? refsRes.data.refs : [],
        });
      } catch {
        if (cancelled) return;
        setPhase({
          kind: "error",
          message: "Can't reach Octanest. Check your connection and try again.",
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, parsed.ref, parsed.path]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-10">
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-48 w-full" />
        </div>
      } @else {
        @if (phase.kind === "error") {
          <p role="alert" className="text-[16px] text-destructive">
            {phase.message as string}
          </p>
        } @else {
          <>
            <RepoChrome repo={phase.repo} active="code" />
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
    </div>
  }
}
