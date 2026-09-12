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
import { CloneBox } from "@/components/repo/clone-box";
import { QuickSetup } from "@/components/repo/quick-setup";
import { ReadmePanel } from "@/components/repo/readme-panel";
import { RefSelect } from "@/components/repo/ref-select";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { findReadmeName, shortRefName } from "@/lib/repo-browse";

export const Route = createFileRoute("/$owner/$repo/")({
  component: RepoCodeHome,
  head: ({ params }) => ({
    meta: [
      {
        title: `${params.owner}/${params.repo} · Octanest`,
      },
    ],
  }),
});

type BrowsePhase =
  | { kind: "loading" }
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "empty";
      repo: RepoPublic;
    }
  | {
      kind: "ready";
      repo: RepoPublic;
      refName: string;
      entries: RepoTreeEntry[];
      refs: RepoRefEntry[];
      readme: { name: string; markdown: string } | null;
    };

/** Code home — Quick setup when empty; tree + README when populated (D-15). */
export function RepoCodeHome() @{
  const params = useParams({ from: "/$owner/$repo/" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const [phase, setPhase] = useState<BrowsePhase>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setPhase({ kind: "loading" });
      try {
        const got = await apiClient.repo.get({ owner, name: repoName });
        if (cancelled) return;
        if (!got.ok) {
          if (got.error.code === "repo.not_found") {
            setPhase({ kind: "not_found" });
            return;
          }
          setPhase({
            kind: "error",
            message: got.error.message || "Could not load repository.",
          });
          return;
        }
        const repo = got.data;
        const refName = repo.default_branch || "main";
        const [treeRes, refsRes] = await Promise.all([
          apiClient.repo.tree({
            owner,
            name: repoName,
            ref: refName,
            path: "",
          }),
          apiClient.repo.refs({ owner, name: repoName }),
        ]);
        if (cancelled) return;
        if (!treeRes.ok) {
          if (treeRes.error.code === "repo.not_found") {
            setPhase({ kind: "not_found" });
            return;
          }
          setPhase({
            kind: "error",
            message: treeRes.error.message || "Could not load tree.",
          });
          return;
        }
        const refs = refsRes.ok ? refsRes.data.refs : [];
        if (treeRes.data.empty) {
          setPhase({ kind: "empty", repo });
          return;
        }
        const entries = treeRes.data.entries;
        const readmeName = findReadmeName(entries);
        let readme: { name: string; markdown: string } | null = null;
        if (readmeName) {
          const blobRes = await apiClient.repo.blob({
            owner,
            name: repoName,
            ref: refName,
            path: readmeName,
          });
          if (
            !cancelled &&
            blobRes.ok &&
            !blobRes.data.is_binary &&
            blobRes.data.encoding === "utf-8" &&
            blobRes.data.content
          ) {
            readme = { name: readmeName, markdown: blobRes.data.content };
          }
        }
        if (cancelled) return;
        setPhase({
          kind: "ready",
          repo,
          refName: shortRefName(treeRes.data.ref) || refName,
          entries,
          refs,
          readme,
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
  }, [owner, repoName]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-10">
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-10 w-full" />
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
            @if (phase.kind === "empty") {
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
              </>
            } @else {
              <>
                <div className="mb-4 flex flex-wrap items-center gap-3">
                  <RefSelect
                    owner={owner}
                    repo={repoName}
                    refs={phase.refs}
                    value={phase.refName}
                    mode="home"
                  />
                  <div className="ml-auto">
                    <CloneBox
                      owner={owner}
                      repo={repoName}
                      refName={phase.refName}
                      empty={false}
                    />
                  </div>
                </div>
                <FileTree
                  owner={owner}
                  repo={repoName}
                  refName={phase.refName}
                  basePath=""
                  entries={phase.entries}
                />
                @if (phase.readme) {
                  <ReadmePanel
                    markdown={phase.readme.markdown}
                    fileName={phase.readme.name}
                  />
                }
              </>
            }
          </>
        }
      }
    </div>
  }
}
