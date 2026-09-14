import {
  createFileRoute,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import type { IssuePublic, RepoPublic } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { apiClient } from "@/lib/api-client";
import { renderGfm } from "@/lib/markdown";
import type { RepoLayoutLoaderData } from "@/lib/repo-store";
import { fetchIssueGet, fetchRepoGet } from "@/lib/ssr-repo";

export type IssueDetailLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "ready"; repo: RepoPublic; issue: IssuePublic };

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

async function loadIssueDetail(
  owner: string,
  repoName: string,
  number: number,
): Promise<IssueDetailLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const issueRes = await fetchIssueGet({
    data: { owner, name: repoName, number },
  });
  if (!issueRes.ok) {
    if (
      issueRes.error.code === "repo.not_found" ||
      issueRes.error.code === "issue.not_found"
    ) {
      return { kind: "not_found" };
    }
    return {
      kind: "error",
      message: issueRes.error.message || "Could not load issue.",
    };
  }
  return { kind: "ready", repo: got.data, issue: issueRes.data };
}

export const Route = createFileRoute("/$owner/$repo/issues/$n")({
  component: IssueDetailPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Issue #${(params as { n: string }).n} · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<IssueDetailLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const n = Number((params as { n?: string }).n ?? "0");
    try {
      return await loadIssueDetail(owner, repoName, n);
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

/** Issue detail — title/body + empty shells for later panels (ISS-01 tracer). */
export function IssueDetailPage() @{
  const params = useParams({ strict: false }) as {
    owner?: string;
    repo?: string;
    n?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const number = Number(params.n ?? "0");
  const layout = useLoaderData({ from: "/$owner/$repo" }) as
    | RepoLayoutLoaderData
    | undefined;
  const routeLoader = useLoaderData({ from: "/$owner/$repo/issues/$n" }) as
    | IssueDetailLoaderData
    | undefined;

  const [bodyHtml, setBodyHtml] = useState("");

  const repoQ = useQuery({
    queryKey: ["repo", "get", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.repo.get({ owner, name: repoName });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !!owner && !!repoName,
    initialData:
      routeLoader?.kind === "ready"
        ? routeLoader.repo
        : layout?.status === "ok"
          ? (layout.repo ?? undefined)
          : undefined,
  });

  const issueQ = useQuery({
    queryKey: ["issue", "get", owner, repoName, number] as const,
    queryFn: async () => {
      const res = await apiClient.issue.get({
        owner,
        name: repoName,
        number,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !!owner && !!repoName && number > 0,
    initialData:
      routeLoader?.kind === "ready" ? routeLoader.issue : undefined,
  });

  const repo = repoQ.data ?? null;
  const issue = issueQ.data ?? null;

  useEffect(() => {
    if (!issue) return;
    let cancelled = false;
    (async () => {
      const html = await renderGfm(issue.body || "");
      if (!cancelled) setBodyHtml(html);
    })();
    return () => {
      cancelled = true;
    };
  }, [issue?.id, issue?.body]);

  @if (
    layout?.status === "not_found" ||
    routeLoader?.kind === "not_found" ||
    (!repoQ.isPending && !repo) ||
    (!issueQ.isPending && !issue && !issueQ.isError)
  ) {
    <RepoNotFound />
  } @else if (!repo || !issue) {
    <p className="text-[16px] text-muted-foreground">Loading issue…</p>
  } @else {
    <>
      <RepoChrome repo={repo} active="issues" />
      <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <div className="flex flex-col gap-4">
          <div>
            <p className="text-[13px] text-muted-foreground">
              {`#${issue.number} · ${issue.state}` as string}
            </p>
            <h1
              data-testid="issue-title"
              className="mt-1 text-[24px] font-semibold text-foreground"
            >
              {issue.title as string}
            </h1>
            <p className="mt-1 text-[13px] text-muted-foreground">
              {`opened by @${issue.author_username}` as string}
            </p>
          </div>
          <div
            className="prose prose-sm max-w-none rounded-md border border-border px-4 py-3"
            dangerouslySetInnerHTML={{ __html: bodyHtml }}
          />
          <section aria-labelledby="issue-comments-heading">
            <h2
              id="issue-comments-heading"
              className="text-[16px] font-semibold text-foreground"
            >
              Comments
            </h2>
            <p className="mt-2 text-[14px] text-muted-foreground">
              No comments yet.
            </p>
          </section>
        </div>
        <aside className="flex flex-col gap-6">
          <section aria-labelledby="issue-labels-heading">
            <h2
              id="issue-labels-heading"
              className="text-[14px] font-semibold text-foreground"
            >
              Labels
            </h2>
            <p className="mt-1 text-[13px] text-muted-foreground">None yet</p>
          </section>
          <section aria-labelledby="issue-assignees-heading">
            <h2
              id="issue-assignees-heading"
              className="text-[14px] font-semibold text-foreground"
            >
              Assignees
            </h2>
            <p className="mt-1 text-[13px] text-muted-foreground">None yet</p>
          </section>
          <section aria-labelledby="issue-linked-prs-heading">
            <h2
              id="issue-linked-prs-heading"
              className="text-[14px] font-semibold text-foreground"
            >
              Linked PRs
            </h2>
            <p className="mt-1 text-[13px] text-muted-foreground">
              No linked pull requests yet.
            </p>
          </section>
        </aside>
      </div>
    </>
  }
}

export default IssueDetailPage;
