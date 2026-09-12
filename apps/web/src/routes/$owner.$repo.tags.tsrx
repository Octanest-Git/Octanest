import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import type { RepoPublic, RepoRefEntry } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import {
  archiveTarGzUrl,
  archiveZipUrl,
  isTagRef,
  shortOid,
  shortRefName,
  treeHref,
} from "@/lib/repo-browse";

export const Route = createFileRoute("/$owner/$repo/tags")({
  component: RepoTagsPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Tags · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
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
      tags: RepoRefEntry[];
    };

/** Tags list + archive download only — no create-tag CTA (UI-SPEC lock). */
export function RepoTagsPage() @{
  const params = useParams({ from: "/$owner/$repo/tags" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [reloadKey, setReloadKey] = useState(0);

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
        const refsRes = await apiClient.repo.refs({
          owner,
          name: repoName,
        });
        if (cancelled) return;
        if (!refsRes.ok) {
          setPhase(
            refsRes.error.code === "repo.not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: refsRes.error.message || "Could not load tags.",
                },
          );
          return;
        }
        const tags = refsRes.data.refs
          .filter((r) => isTagRef(r.name))
          .sort((a, b) =>
            shortRefName(a.name).localeCompare(shortRefName(b.name), undefined, {
              sensitivity: "base",
            }),
          );
        setPhase({ kind: "ready", repo: got.data, tags });
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
  }, [owner, repoName, reloadKey]);

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-10">
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      } @else {
        @if (phase.kind === "error") {
          <div className="flex flex-col gap-4">
            <p role="alert" className="text-[16px] text-destructive">
              {phase.message as string}
            </p>
            <Button
              type="button"
              variant="secondary"
              onClick={() => setReloadKey((k) => k + 1)}
            >
              Try again
            </Button>
          </div>
        } @else {
          <>
            <RepoChrome repo={phase.repo} active="tags" />
            <h2 className="mb-4 text-[18px] font-semibold text-foreground">
              Tags
            </h2>
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
    </div>
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
