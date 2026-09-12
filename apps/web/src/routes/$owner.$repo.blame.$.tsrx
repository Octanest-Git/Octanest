import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import type {
  RepoBlameResponse,
  RepoPublic,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { PathBreadcrumb } from "@/components/repo/path-breadcrumb";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { commitHref, parseRefAndPath } from "@/lib/repo-browse";
import { formatRelativeUpdated } from "@/lib/relative-time";

export const Route = createFileRoute("/$owner/$repo/blame/$")({
  component: RepoBlamePage,
  head: ({ params }) => {
    const { path } = parseRefAndPath(
      (params as { _splat?: string })._splat,
    );
    const base = path.split("/").pop() || path || "file";
    const trunc = base.length > 40 ? `…${base.slice(-36)}` : base;
    return {
      meta: [{ title: `Blame · ${trunc} · Octanest` }],
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
      blame: RepoBlameResponse;
    };

function shortSha(sha: string): string {
  return sha.slice(0, 7);
}

function relativeCommitTime(iso: string): string {
  return formatRelativeUpdated(iso).replace(/^Updated\s+/i, "");
}

export function RepoBlamePage() @{
  const params = useParams({ from: "/$owner/$repo/blame/$" }) as {
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
        if (!parsed.path) {
          setPhase({ kind: "not_found" });
          return;
        }
        const blameRes = await apiClient.repo.blame({
          owner,
          name: repoName,
          ref: refName,
          path: parsed.path,
        });
        if (cancelled) return;
        if (!blameRes.ok) {
          setPhase(
            blameRes.error.code === "repo.not_found" ||
              blameRes.error.code === "repo.path_not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: blameRes.error.message || "Could not load blame.",
                },
          );
          return;
        }
        setPhase({
          kind: "ready",
          repo,
          refName: blameRes.data.ref || refName,
          path: blameRes.data.path || parsed.path,
          blame: blameRes.data,
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
          <Skeleton className="h-96 w-full" />
        </div>
      } @else {
        @if (phase.kind === "error") {
          <p role="alert" className="text-[16px] text-destructive">
            {phase.message as string}
          </p>
        } @else {
          <>
            <RepoChrome repo={phase.repo} active="code" />
            <PathBreadcrumb
              owner={owner}
              repo={repoName}
              refName={phase.refName}
              path={phase.path}
            />
            @if (phase.blame.truncated) {
              <p className="mb-2 text-[14px] text-muted-foreground">
                Blame truncated for size.
              </p>
            }
            <div className="overflow-x-auto border border-border">
              <table className="w-full border-collapse text-left text-[14px] leading-[1.45]">
                <tbody>
                  @for (const line of phase.blame.lines; key line.line_number) {
                    <tr className="border-b border-border/60 align-top">
                      <td className="whitespace-nowrap bg-muted/30 px-2 py-0.5 font-mono text-[12px] text-muted-foreground">
                        <a
                          href={commitHref(owner, repoName, line.sha) as string}
                          className="text-primary no-underline hover:underline"
                          title={line.sha as string}
                        >
                          {shortSha(line.sha) as string}
                        </a>
                        <span className="ml-2 text-muted-foreground">
                          {line.author_name as string}
                        </span>
                        <time
                          className="ml-2 text-muted-foreground"
                          dateTime={line.authored_at as string}
                        >
                          {relativeCommitTime(line.authored_at) as string}
                        </time>
                      </td>
                      <td className="select-none px-2 py-0.5 font-mono text-[12px] text-muted-foreground">
                        {String(line.line_number) as string}
                      </td>
                      <td className="px-2 py-0.5 font-mono text-[14px] text-foreground">
                        <pre className="m-0 whitespace-pre-wrap font-mono">
                          {line.content as string}
                        </pre>
                      </td>
                    </tr>
                  }
                </tbody>
              </table>
            </div>
          </>
        }
      }
    </div>
  }
}
