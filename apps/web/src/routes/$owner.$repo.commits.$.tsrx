import { createFileRoute, useLoaderData, useParams } from "@octanejs/tanstack-router";
import type { RepoCommitSummary, RepoPublic } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { apiClient } from "@/lib/api-client";
import { commitHref } from "@/lib/repo-browse";
import { formatRelativeUpdated } from "@/lib/relative-time";
import { fetchRepoCommits, fetchRepoGet } from "@/lib/ssr-repo";

export type CommitsLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      refName: string;
      commits: RepoCommitSummary[];
      skip: number;
      limit: number;
    };

type Phase =
  | { kind: "loading" }
  | CommitsLoaderData
  | {
      kind: "ready";
      repo: RepoPublic;
      refName: string;
      commits: RepoCommitSummary[];
      skip: number;
      limit: number;
      loadingMore: boolean;
    };

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

function toReadyPhase(
  data: Extract<CommitsLoaderData, { kind: "ready" }>,
): Extract<Phase, { kind: "ready" }> {
  return { ...data, loadingMore: false };
}

async function loadCommitsPage(
  owner: string,
  repoName: string,
  refFromUrl: string,
): Promise<CommitsLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const repo = got.data;
  const refName = refFromUrl || repo.default_branch || "main";
  const list = await fetchRepoCommits({
    data: { owner, name: repoName, ref: refName, skip: 0, limit: 30 },
  });
  if (!list.ok) {
    if (list.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: list.error.message || "Could not load commits.",
    };
  }
  return {
    kind: "ready",
    repo,
    refName: list.data.ref || refName,
    commits: list.data.commits,
    skip: list.data.skip,
    limit: list.data.limit,
  };
}

export const Route = createFileRoute("/$owner/$repo/commits/$")({
  component: RepoCommitsPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Commits · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<CommitsLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const splat = String((params as { _splat?: string })._splat ?? "");
    const refFromUrl = decodeURIComponent(splat.split("/")[0] ?? "");
    try {
      return await loadCommitsPage(owner, repoName, refFromUrl);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

function relativeCommitTime(iso: string): string {
  return formatRelativeUpdated(iso).replace(/^Updated\s+/i, "");
}

export function RepoCommitsPage() @{
  const params = useParams({ from: "/$owner/$repo/commits/$" }) as {
    owner: string;
    repo: string;
    _splat?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const refFromUrl = decodeURIComponent((params._splat ?? "").split("/")[0] ?? "");
  const loaderData = useLoaderData({ from: "/$owner/$repo/commits/$" }) as
    | CommitsLoaderData
    | undefined;

  const [phase, setPhase] = useState<Phase>(() =>
    loaderData
      ? loaderData.kind === "ready"
        ? toReadyPhase(loaderData)
        : loaderData
      : { kind: "loading" },
  );

  useEffect(() => {
    if (loaderData) {
      setPhase(loaderData.kind === "ready" ? toReadyPhase(loaderData) : loaderData);
      return;
    }
    let cancelled = false;
    void (async () => {
      setPhase({ kind: "loading" });
      try {
        const next = await loadCommitsPage(owner, repoName, refFromUrl);
        if (cancelled) return;
        setPhase(next.kind === "ready" ? toReadyPhase(next) : next);
      } catch {
        if (!cancelled) setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, refFromUrl, loaderData]);

  async function loadMore() {
    if (phase.kind !== "ready" || phase.loadingMore) return;
    const nextSkip = phase.skip + phase.limit;
    setPhase({ ...phase, loadingMore: true });
    try {
      const list = await apiClient.repo.commits({
        owner,
        name: repoName,
        ref: phase.refName,
        skip: nextSkip,
        limit: phase.limit,
      });
      if (!list.ok) {
        setPhase({ ...phase, loadingMore: false });
        return;
      }
      setPhase({
        ...phase,
        commits: [...phase.commits, ...list.data.commits],
        skip: list.data.skip,
        loadingMore: false,
      });
    } catch {
      setPhase({ ...phase, loadingMore: false });
    }
  }

  @if (phase.kind === "not_found") {
    <RepoNotFound />;
  } @else if (phase.kind === "loading") {
    <p className="text-[16px] text-muted-foreground">Loading commits…</p>;
  } @else if (phase.kind === "error") {
    <p role="alert" className="text-[16px] text-destructive">
      {phase.message as string}
    </p>;
  } @else {
    <>
      <h2 className="mb-4 text-[18px] font-semibold text-foreground">
        Commits on {phase.refName as string}
      </h2>
      @if (phase.commits.length === 0) {
        <p className="text-[16px] text-muted-foreground">No commits yet on this ref.</p>;
      } @else {
        <>
          <ul className="divide-y divide-border border-y border-border">
            @for (const c of phase.commits; key c.sha) {
              <li className="flex flex-wrap items-baseline gap-x-4 gap-y-1 py-3">
                <a
                  href={commitHref(owner, repoName, c.sha) as string}
                  className="font-mono text-[14px] text-primary no-underline hover:underline"
                  title={c.sha as string}
                >
                  {c.short_sha as string}
                </a>
                <a
                  href={commitHref(owner, repoName, c.sha) as string}
                  className="min-w-0 flex-1 truncate text-[16px] text-foreground no-underline hover:underline"
                  title={c.subject as string}
                >
                  {c.subject as string}
                </a>
                <span className="text-[14px] text-muted-foreground">
                  {c.author_name as string}
                </span>
                <time
                  className="text-[14px] text-muted-foreground"
                  dateTime={c.authored_at as string}
                >
                  {relativeCommitTime(c.authored_at) as string}
                </time>
              </li>;
            }
          </ul>
          @if (phase.commits.length >= phase.limit) {
            <div className="mt-4">
              <button
                type="button"
                className="text-[14px] font-medium text-primary hover:underline"
                disabled={phase.loadingMore}
                onClick={() => {
                  void loadMore();
                }}
              >
                {phase.loadingMore ? "Loading…" : "Load more"}
              </button>
            </div>;
          }
        </>;
      }
    </>;
  }
}
