import {
  createFileRoute,
  useLoaderData,
  useParams,
  useRouter,
} from "@octanejs/tanstack-router";
import type { RepoPublic, RepoRefEntry } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Button } from "@/components/ui/button";
import {
  archiveTarGzUrl,
  archiveZipUrl,
  isTagRef,
  shortOid,
  shortRefName,
  treeHref,
} from "@/lib/repo-browse";
import { fetchRepoGet, fetchRepoRefs } from "@/lib/ssr-repo";

export type TagsLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "ready"; repo: RepoPublic; tags: RepoRefEntry[] };

type Phase = { kind: "loading" } | TagsLoaderData;

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

function sortTags(refs: RepoRefEntry[]): RepoRefEntry[] {
  return refs
    .filter((r) => isTagRef(r.name))
    .sort((a, b) =>
      shortRefName(a.name).localeCompare(shortRefName(b.name), undefined, {
        sensitivity: "base",
      }),
    );
}

async function loadTagsPage(
  owner: string,
  repoName: string,
): Promise<TagsLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const refsRes = await fetchRepoRefs({ data: { owner, name: repoName } });
  if (!refsRes.ok) {
    if (refsRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: refsRes.error.message || "Could not load tags.",
    };
  }
  return {
    kind: "ready",
    repo: got.data,
    tags: sortTags(refsRes.data.refs),
  };
}

export const Route = createFileRoute("/$owner/$repo/tags")({
  component: RepoTagsPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Tags · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<TagsLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    try {
      return await loadTagsPage(owner, repoName);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

/** Tags list + archive download only — no create-tag CTA (UI-SPEC lock). */
export function RepoTagsPage() @{
  const params = useParams({ from: "/$owner/$repo/tags" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const router = useRouter();
  const loaderData = useLoaderData({ from: "/$owner/$repo/tags" }) as
    | TagsLoaderData
    | undefined;

  const [phase, setPhase] = useState<Phase>(() => loaderData ?? { kind: "loading" });

  useEffect(() => {
    if (loaderData) {
      setPhase(loaderData);
      return;
    }
    let cancelled = false;
    (async () => {
      setPhase({ kind: "loading" });
      try {
        const next = await loadTagsPage(owner, repoName);
        if (!cancelled) setPhase(next);
      } catch {
        if (!cancelled) setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, loaderData]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else if (phase.kind === "loading") {
    <p className="text-[16px] text-muted-foreground">Loading tags…</p>
  } @else if (phase.kind === "error") {
    <div className="flex flex-col gap-4">
      <p role="alert" className="text-[16px] text-destructive">
        {phase.message as string}
      </p>
      <Button
        type="button"
        variant="secondary"
        onClick={() => void router.invalidate()}
      >
        Try again
      </Button>
    </div>
  } @else {
    <>
      <h2 className="mb-4 text-[18px] font-semibold text-foreground">Tags</h2>
      @if (phase.tags.length === 0) {
        <p className="text-[16px] text-muted-foreground">No tags yet.</p>
      } @else {
        <ul className="divide-y divide-border border-y border-border">
          @for (const t of phase.tags; key t.name) {
            <TagRow owner={owner} repo={repoName} tag={t} />
          }
        </ul>
      }
    </>
  }
}

function TagRow({
  owner,
  repo,
  tag,
}: {
  owner: string;
  repo: string;
  tag: RepoRefEntry;
}) @{
  const name = shortRefName(tag.name);
  const tip = shortOid(tag.oid);
  const href = treeHref(owner, repo, name, "");
  const zipHref = archiveZipUrl(owner, repo, name);
  const tarHref = archiveTarGzUrl(owner, repo, name);

  <li className="flex flex-wrap items-center gap-x-4 gap-y-2 py-3">
    <div className="min-w-0 flex-1">
      <a
        href={href as string}
        className="block truncate text-[16px] font-semibold text-primary no-underline hover:underline"
        title={name as string}
      >
        {name as string}
      </a>
      <p
        className="mt-0.5 font-mono text-[13px] text-muted-foreground"
        title={tag.oid as string}
      >
        {tip as string}
      </p>
    </div>
    <div className="flex flex-wrap gap-2">
      <Button
        type="button"
        variant="secondary"
        onClick={() => {
          window.location.assign(zipHref);
        }}
      >
        Download ZIP
      </Button>
      <Button
        type="button"
        variant="secondary"
        onClick={() => {
          window.location.assign(tarHref);
        }}
      >
        Download tar.gz
      </Button>
    </div>
  </li>
}
