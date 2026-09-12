import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import type {
  RepoCompareResponse,
  RepoPublic,
  RepoRefEntry,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { compareHref, shortRefName } from "@/lib/repo-browse";

export const Route = createFileRoute("/$owner/$repo/compare/$")({
  component: RepoComparePage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Compare · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
});

function parseCompareSplat(splat: string | undefined): {
  base: string;
  head: string;
} {
  const s = splat ?? "";
  const idx = s.indexOf("...");
  if (idx === -1) {
    return { base: decodeURIComponent(s), head: "" };
  }
  return {
    base: decodeURIComponent(s.slice(0, idx)),
    head: decodeURIComponent(s.slice(idx + 3)),
  };
}

type Phase =
  | { kind: "loading" }
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      refs: RepoRefEntry[];
      base: string;
      head: string;
      compare: RepoCompareResponse | null;
    };

export function RepoComparePage() @{
  const params = useParams({ from: "/$owner/$repo/compare/$" }) as {
    owner: string;
    repo: string;
    _splat?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const parsed = parseCompareSplat(params._splat);
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [baseDraft, setBaseDraft] = useState(parsed.base);
  const [headDraft, setHeadDraft] = useState(parsed.head);

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
        const refsRes = await apiClient.repo.refs({ owner, name: repoName });
        const refs = refsRes.ok ? refsRes.data.refs : [];
        const base = parsed.base || repo.default_branch || "main";
        const head = parsed.head || repo.default_branch || "main";
        setBaseDraft(base);
        setHeadDraft(head);

        if (!parsed.head) {
          setPhase({
            kind: "ready",
            repo,
            refs,
            base,
            head,
            compare: null,
          });
          return;
        }

        const cmp = await apiClient.repo.compare({
          owner,
          name: repoName,
          base,
          head,
        });
        if (cancelled) return;
        if (!cmp.ok) {
          setPhase(
            cmp.error.code === "repo.not_found" ||
              cmp.error.code === "repo.path_not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: cmp.error.message || "Could not compare refs.",
                },
          );
          return;
        }
        setPhase({
          kind: "ready",
          repo,
          refs,
          base,
          head,
          compare: cmp.data,
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
  }, [owner, repoName, parsed.base, parsed.head]);

  function goCompare() {
    const b = baseDraft.trim() || "main";
    const h = headDraft.trim() || "main";
    if (typeof window !== "undefined") {
      window.location.href = compareHref(owner, repoName, b, h);
    }
  }

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
            <RepoChrome repo={phase.repo} active="commits" />
            <h2 className="mb-4 text-[18px] font-semibold text-foreground">
              Compare changes
            </h2>
            <div className="mb-6 flex flex-wrap items-end gap-3">
              <label className="flex min-w-0 flex-col gap-1 text-[14px] text-muted-foreground">
                Base
                <input
                  className="max-w-[14rem] truncate rounded-md border border-input bg-card px-3 py-2 font-mono text-[14px] text-foreground"
                  value={baseDraft as string}
                  list="octanest-compare-refs"
                  onInput={(e) => {
                    setBaseDraft((e.currentTarget as HTMLInputElement).value);
                  }}
                />
              </label>
              <span className="pb-2 text-muted-foreground">...</span>
              <label className="flex min-w-0 flex-col gap-1 text-[14px] text-muted-foreground">
                Head
                <input
                  className="max-w-[14rem] truncate rounded-md border border-input bg-card px-3 py-2 font-mono text-[14px] text-foreground"
                  value={headDraft as string}
                  list="octanest-compare-refs"
                  onInput={(e) => {
                    setHeadDraft((e.currentTarget as HTMLInputElement).value);
                  }}
                />
              </label>
              <datalist id="octanest-compare-refs">
                @for (const r of phase.refs; key r.name) {
                  <option value={shortRefName(r.name) as string} />
                }
              </datalist>
              <button
                type="button"
                className="rounded-md bg-primary px-3 py-2 text-[14px] font-medium text-primary-foreground"
                onClick={() => {
                  goCompare();
                }}
              >
                Compare
              </button>
            </div>
            @if (!phase.compare) {
              <p className="text-[16px] text-muted-foreground">
                Choose base and head refs to compare.
              </p>
            } @else {
              @if (phase.compare.empty) {
                <p className="text-[16px] text-muted-foreground">
                  Nothing to compare for these refs.
                </p>
              } @else {
                <>
                  @if (phase.compare.truncated) {
                    <p className="mb-3 text-[14px] text-muted-foreground">
                      Diff truncated for size.
                    </p>
                  }
                  <div className="flex flex-col gap-6">
                    @for (const f of phase.compare.files; key f.path) {
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
                </>
              }
            }
          </>
        }
      }
    </div>
  }
}
