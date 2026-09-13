import {
  createFileRoute,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import type {
  RepoCompareResponse,
  RepoPublic,
  RepoRefEntry,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { DiffPatch } from "@/components/repo/diff-patch";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { compareHref, shortRefName } from "@/lib/repo-browse";
import {
  fetchRepoCompare,
  fetchRepoGet,
  fetchRepoRefs,
} from "@/lib/ssr-repo";

export type CompareLoaderData =
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

type Phase = { kind: "loading" } | CompareLoaderData;

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

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

async function loadComparePage(
  owner: string,
  repoName: string,
  splat: string | undefined,
): Promise<CompareLoaderData> {
  const parsed = parseCompareSplat(splat);
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
  const base = parsed.base || repo.default_branch || "main";
  const head = parsed.head || repo.default_branch || "main";

  if (!parsed.head) {
    return {
      kind: "ready",
      repo,
      refs,
      base,
      head,
      compare: null,
    };
  }

  const cmp = await fetchRepoCompare({
    data: { owner, name: repoName, base, head },
  });
  if (!cmp.ok) {
    if (
      cmp.error.code === "repo.not_found" ||
      cmp.error.code === "repo.path_not_found"
    ) {
      return { kind: "not_found" };
    }
    return {
      kind: "error",
      message: cmp.error.message || "Could not compare refs.",
    };
  }
  return {
    kind: "ready",
    repo,
    refs,
    base,
    head,
    compare: cmp.data,
  };
}

export const Route = createFileRoute("/$owner/$repo/compare/$")({
  component: RepoComparePage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Compare · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<CompareLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const splat = (params as { _splat?: string })._splat;
    try {
      return await loadComparePage(owner, repoName, splat);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

export function RepoComparePage() @{
  const params = useParams({ from: "/$owner/$repo/compare/$" }) as {
    owner: string;
    repo: string;
    _splat?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const parsed = parseCompareSplat(params._splat);
  const loaderData = useLoaderData({ from: "/$owner/$repo/compare/$" }) as
    | CompareLoaderData
    | undefined;

  const [phase, setPhase] = useState<Phase>(() =>
    loaderData ?? { kind: "loading" },
  );
  const [baseDraft, setBaseDraft] = useState(() =>
    loaderData?.kind === "ready" ? loaderData.base : parsed.base,
  );
  const [headDraft, setHeadDraft] = useState(() =>
    loaderData?.kind === "ready" ? loaderData.head : parsed.head,
  );

  useEffect(() => {
    if (loaderData) {
      setPhase(loaderData);
      if (loaderData.kind === "ready") {
        setBaseDraft(loaderData.base);
        setHeadDraft(loaderData.head);
      }
      return;
    }
    let cancelled = false;
    (async () => {
      setPhase({ kind: "loading" });
      try {
        const next = await loadComparePage(owner, repoName, params._splat);
        if (cancelled) return;
        setPhase(next);
        if (next.kind === "ready") {
          setBaseDraft(next.base);
          setHeadDraft(next.head);
        }
      } catch {
        if (!cancelled) setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, parsed.base, parsed.head, loaderData, params._splat]);

  function goCompare() {
    const b = baseDraft.trim() || "main";
    const h = headDraft.trim() || "main";
    if (typeof window !== "undefined") {
      window.location.href = compareHref(owner, repoName, b, h);
    }
  }

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else if (phase.kind === "loading") {
    <p className="text-[16px] text-muted-foreground">Loading compare…</p>
  } @else if (phase.kind === "error") {
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
      } @else if (phase.compare.empty) {
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
              <DiffPatch
                path={f.path as string}
                status={f.status as string}
                patch={f.patch as string}
              />
            }
          </div>
        </>
      }
    </>
  }
}
