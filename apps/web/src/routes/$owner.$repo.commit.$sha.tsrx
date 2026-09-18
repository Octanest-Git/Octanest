import { createFileRoute, useLoaderData, useParams } from "@octanejs/tanstack-router";
import type { RepoCommitResponse, RepoPublic } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { FileDiff } from "@octanejs/lucide";
import { DiffPatch } from "@/components/repo/diff-patch";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { EmptyState } from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { formatRelativeUpdated } from "@/lib/relative-time";
import { fetchRepoCommit, fetchRepoGet } from "@/lib/ssr-repo";

export type CommitLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "ready"; repo: RepoPublic; commit: RepoCommitResponse };

type Phase = { kind: "loading" } | CommitLoaderData;

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

async function loadCommitPage(
  owner: string,
  repoName: string,
  sha: string,
): Promise<CommitLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const detail = await fetchRepoCommit({
    data: { owner, name: repoName, sha },
  });
  if (!detail.ok) {
    if (detail.error.code === "repo.not_found" || detail.error.code === "repo.path_not_found") {
      return { kind: "not_found" };
    }
    return {
      kind: "error",
      message: detail.error.message || "Could not load commit.",
    };
  }
  return { kind: "ready", repo: got.data, commit: detail.data };
}

export const Route = createFileRoute("/$owner/$repo/commit/$sha")({
  component: RepoCommitPage,
  head: ({ params }) => {
    const sha = (params as { sha: string }).sha ?? "";
    const short = sha.slice(0, 7) || "commit";
    return {
      meta: [
        {
          title: `${short} · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
        },
      ],
    };
  },
  loader: async ({ params }): Promise<CommitLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const sha = String((params as { sha?: string }).sha ?? "");
    try {
      return await loadCommitPage(owner, repoName, sha);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

function relativeCommitTime(iso: string): string {
  return formatRelativeUpdated(iso).replace(/^Updated\s+/i, "");
}

export function RepoCommitPage() @{
  const params = useParams({ from: "/$owner/$repo/commit/$sha" }) as {
    owner: string;
    repo: string;
    sha: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const sha = params.sha ?? "";
  const loaderData = useLoaderData({ from: "/$owner/$repo/commit/$sha" }) as
    | CommitLoaderData
    | undefined;

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
        const next = await loadCommitPage(owner, repoName, sha);
        if (!cancelled) setPhase(next);
      } catch {
        if (!cancelled) setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, sha, loaderData]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />;
  } @else if (phase.kind === "loading") {
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-2 border-b border-border pb-4">
        <Skeleton className="h-6 w-2/3" />
        <Skeleton className="h-4 w-48" />
      </div>
      <Skeleton className="h-40 w-full" />
    </div>;
  } @else if (phase.kind === "error") {
    <p role="alert" className="text-[16px] text-destructive">
      {phase.message as string}
    </p>;
  } @else {
    <>
      <header className="mb-6 border-b border-border pb-4">
        <h2 className="text-[20px] font-semibold text-foreground">
          {phase.commit.subject as string}
        </h2>
        @if (phase.commit.body.trim()) {
          <pre className="mt-2 whitespace-pre-wrap font-[family-name:var(--font-sans)] text-[16px] text-muted-foreground">
            {phase.commit.body as string}
          </pre>;
        }
        <p className="mt-3 text-[14px] text-muted-foreground">
          <span className="font-medium text-foreground">{phase.commit.author_name as string}</span>
          {" · "}
          <time dateTime={phase.commit.authored_at as string}>
            {relativeCommitTime(phase.commit.authored_at) as string}
          </time>
          {" · "}
          <code className="font-mono text-[13px]">{phase.commit.short_sha as string}</code>
        </p>
        @if (phase.commit.truncated) {
          <p className="mt-2 text-[14px] text-muted-foreground">Diff truncated for size.</p>;
        }
      </header>
      @if (phase.commit.files.length === 0) {
        <EmptyState
          icon={<FileDiff aria-hidden size={22} />}
          title="No file changes"
          description="This commit doesn't touch any files (it may be an empty or merge commit)."
        />;
      } @else {
        <div className="flex flex-col gap-6">
          @for (const f of phase.commit.files; key f.path) {
            <DiffPatch
              path={f.path as string}
              status={f.status as string}
              patch={f.patch as string}
            />;
          }
        </div>;
      }
    </>;
  }
}
