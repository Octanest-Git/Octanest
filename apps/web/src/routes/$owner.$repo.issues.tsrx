import {
  createFileRoute,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import type { IssueListResponse, RepoPublic } from "@octanest/api-client";
import { useState } from "octane";
import {
  ISSUE_LIST_PAGE_SIZE,
  IssuesList,
  type IssueListFilters,
  type IssueListStateFilter,
} from "@/components/repo/issues-list";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { apiClient } from "@/lib/api-client";
import type { RepoLayoutLoaderData } from "@/lib/repo-store";
import { fetchIssueList, fetchRepoGet } from "@/lib/ssr-repo";

export type IssuesLoaderData =
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      list: IssueListResponse;
      state: IssueListStateFilter;
    };

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

async function loadIssuesPage(
  owner: string,
  repoName: string,
  state: IssueListStateFilter,
): Promise<IssuesLoaderData> {
  const got = await fetchRepoGet({ data: { owner, name: repoName } });
  if (!got.ok) {
    if (got.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: got.error.message || "Could not load repository.",
    };
  }
  const listRes = await fetchIssueList({
    data: {
      owner,
      name: repoName,
      state,
      offset: 0,
      limit: ISSUE_LIST_PAGE_SIZE,
    },
  });
  if (!listRes.ok) {
    if (listRes.error.code === "repo.not_found") return { kind: "not_found" };
    return {
      kind: "error",
      message: listRes.error.message || "Could not load issues.",
    };
  }
  return {
    kind: "ready",
    repo: got.data,
    list: listRes.data,
    state,
  };
}

export const Route = createFileRoute("/$owner/$repo/issues")({
  component: IssuesListPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Issues · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }): Promise<IssuesLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    try {
      return await loadIssuesPage(owner, repoName, "open");
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

function emptyFilters(state: IssueListStateFilter = "open"): IssueListFilters {
  return {
    state,
    author: "",
    label: "",
    assignee: "",
    q: "",
    offset: 0,
    limit: ISSUE_LIST_PAGE_SIZE,
  };
}

/** Issues list — Open default + filters + offset pages (D-ISS-16..19). */
export function IssuesListPage() @{
  const params = useParams({ strict: false }) as {
    owner?: string;
    repo?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const layout = useLoaderData({ from: "/$owner/$repo" }) as
    | RepoLayoutLoaderData
    | undefined;
  const routeLoader = useLoaderData({ from: "/$owner/$repo/issues" }) as
    | IssuesLoaderData
    | undefined;

  const [filters, setFilters] = useState<IssueListFilters>(() =>
    emptyFilters("open"),
  );
  const [draftAuthor, setDraftAuthor] = useState("");
  const [draftLabel, setDraftLabel] = useState("");
  const [draftAssignee, setDraftAssignee] = useState("");
  const [draftQ, setDraftQ] = useState("");

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

  const labelsQ = useQuery({
    queryKey: ["label", "listForRepo", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.label.listForRepo({
        owner,
        name: repoName,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data.labels;
    },
    enabled: !!owner && !!repoName,
  });

  const listQ = useQuery({
    queryKey: [
      "issue",
      "list",
      owner,
      repoName,
      filters.state,
      filters.author,
      filters.label,
      filters.assignee,
      filters.q,
      filters.offset,
      filters.limit,
    ] as const,
    queryFn: async () => {
      const res = await apiClient.issue.list({
        owner,
        name: repoName,
        state: filters.state,
        author: filters.author || null,
        label: filters.label || null,
        assignee: filters.assignee || null,
        q: filters.q || null,
        offset: filters.offset,
        limit: filters.limit,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !!owner && !!repoName,
    initialData:
      routeLoader?.kind === "ready" &&
      filters.state === "open" &&
      !filters.author &&
      !filters.label &&
      !filters.assignee &&
      !filters.q &&
      filters.offset === 0
        ? routeLoader.list
        : undefined,
  });

  const repo = repoQ.data ?? null;
  const notFound =
    layout?.status === "not_found" ||
    routeLoader?.kind === "not_found" ||
    (repoQ.isError &&
      String(repoQ.error?.message ?? "").includes("not found"));

  const applyFilters = () => {
    setFilters((prev) => ({
      ...prev,
      author: draftAuthor.trim(),
      label: draftLabel.trim(),
      assignee: draftAssignee.trim(),
      q: draftQ.trim(),
      offset: 0,
    }));
  };

  const setState = (next: IssueListStateFilter) => {
    setFilters((prev) => ({ ...prev, state: next, offset: 0 }));
  };

  const prevPage = () => {
    setFilters((prev) => ({
      ...prev,
      offset: Math.max(0, prev.offset - prev.limit),
    }));
  };

  const nextPage = () => {
    setFilters((prev) => ({
      ...prev,
      offset: prev.offset + prev.limit,
    }));
  };

  @if (notFound || (!repoQ.isPending && !repo)) {
    <RepoNotFound />
  } @else if (!repo) {
    <p className="text-[16px] text-muted-foreground">Loading issues…</p>
  } @else {
    <>
      <RepoChrome repo={repo} active="issues" />
      @if (listQ.isError) {
        <p role="alert" className="text-[16px] text-destructive">
          {(listQ.error?.message || NETWORK_ERROR) as string}
        </p>
      } @else {
        <IssuesList
          owner={owner}
          repoName={repoName}
          repo={repo}
          issues={listQ.data?.issues ?? []}
          total={listQ.data?.total ?? 0}
          filters={filters}
          labels={labelsQ.data ?? []}
          draftAuthor={draftAuthor}
          draftLabel={draftLabel}
          draftAssignee={draftAssignee}
          draftQ={draftQ}
          onStateChange={setState}
          onDraftAuthor={setDraftAuthor}
          onDraftLabel={setDraftLabel}
          onDraftAssignee={setDraftAssignee}
          onDraftQ={setDraftQ}
          onApplyFilters={applyFilters}
          onPrevPage={prevPage}
          onNextPage={nextPage}
          pending={listQ.isPending}
        />
      }
    </>
  }
}

export default IssuesListPage;
