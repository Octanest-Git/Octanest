import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import type {
  RepoCommitResponse,
  RepoPublic,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { formatRelativeUpdated } from "@/lib/relative-time";

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
});

type Phase =
  | { kind: "loading" }
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "ready"; repo: RepoPublic; commit: RepoCommitResponse };

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
        const detail = await apiClient.repo.commit({
          owner,
          name: repoName,
          sha,
        });
        if (cancelled) return;
        if (!detail.ok) {
          setPhase(
            detail.error.code === "repo.not_found" ||
              detail.error.code === "repo.path_not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: detail.error.message || "Could not load commit.",
                },
          );
          return;
        }
        setPhase({ kind: "ready", repo: got.data, commit: detail.data });
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
  }, [owner, repoName, sha]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-10">
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-24 w-full" />
          <Skeleton className="h-64 w-full" />
        </div>
      } @else {
        @if (phase.kind === "error") {
          <p role="alert" className="text-[16px] text-destructive">
            {phase.message as string}
          </p>
        } @else {
          <>
            <RepoChrome repo={phase.repo} active="commits" />
            <header className="mb-6 border-b border-border pb-4">
              <h2 className="text-[20px] font-semibold text-foreground">
                {phase.commit.subject as string}
              </h2>
              @if (phase.commit.body.trim()) {
                <pre className="mt-2 whitespace-pre-wrap font-[family-name:var(--font-sans)] text-[16px] text-muted-foreground">
                  {phase.commit.body as string}
                </pre>
              }
              <p className="mt-3 text-[14px] text-muted-foreground">
                <span className="font-medium text-foreground">
                  {phase.commit.author_name as string}
                </span>
                {" · "}
                <time dateTime={phase.commit.authored_at as string}>
                  {relativeCommitTime(phase.commit.authored_at) as string}
                </time>
                {" · "}
                <code className="font-mono text-[13px]">
                  {phase.commit.short_sha as string}
                </code>
              </p>
              @if (phase.commit.truncated) {
                <p className="mt-2 text-[14px] text-muted-foreground">
                  Diff truncated for size.
                </p>
              }
            </header>
            @if (phase.commit.files.length === 0) {
              <p className="text-[16px] text-muted-foreground">
                No file changes in this commit.
              </p>
            } @else {
              <div className="flex flex-col gap-6">
                @for (const f of phase.commit.files; key f.path) {
                  <section className="overflow-hidden border border-border">
                    <h3 className="border-b border-border bg-muted/40 px-3 py-2 font-mono text-[14px] text-foreground">
                      {f.path as string}
                      <span className="ml-2 text-muted-foreground">
                        ({f.status as string})
                      </span>
                    </h3>
                    <pre className="overflow-x-auto p-3 font-mono text-[14px] leading-[1.45] text-foreground">
                      {f.patch as string}
                    </pre>
                  </section>
                }
              </div>
            }
          </>
        }
      }
    </div>
  }
}
