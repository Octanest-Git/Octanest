import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import type {
  RepoCommitSummary,
  RepoPublic,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { commitHref } from "@/lib/repo-browse";
import { formatRelativeUpdated } from "@/lib/relative-time";

export const Route = createFileRoute("/$owner/$repo/commits/$")({
  component: RepoCommitsPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Commits · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
});

type Phase =
  | { kind: "loading" }
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      refName: string;
      commits: RepoCommitSummary[];
      skip: number;
      limit: number;
      loadingMore: boolean;
    };

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
        const refName = refFromUrl || repo.default_branch || "main";
        const list = await apiClient.repo.commits({
          owner,
          name: repoName,
          ref: refName,
          skip: 0,
          limit: 30,
        });
        if (cancelled) return;
        if (!list.ok) {
          setPhase(
            list.error.code === "repo.not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: list.error.message || "Could not load commits.",
                },
          );
          return;
        }
        setPhase({
          kind: "ready",
          repo,
          refName: list.data.ref || refName,
          commits: list.data.commits,
          skip: list.data.skip,
          limit: list.data.limit,
          loadingMore: false,
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
  }, [owner, repoName, refFromUrl]);

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
        setPhase({
          ...phase,
          loadingMore: false,
        });
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
    <RepoNotFound />
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-10">
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      } @else {
        @if (phase.kind === "error") {
          <p role="alert" className="text-[16px] text-destructive">
            {phase.message as string}
          </p>
        } @else {
          <>
            <RepoChrome repo={phase.repo} active="commits" />
            <h2 className="mb-4 text-[18px] font-semibold text-foreground">
              Commits on {phase.refName as string}
            </h2>
            @if (phase.commits.length === 0) {
              <p className="text-[16px] text-muted-foreground">
                No commits yet on this ref.
              </p>
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
                    </li>
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
                  </div>
                }
              </>
            }
          </>
        }
      }
    </div>
  }
}
