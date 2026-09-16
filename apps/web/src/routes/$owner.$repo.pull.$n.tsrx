import { createFileRoute, useParams } from "@octanejs/tanstack-router";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { PullCommits } from "@/components/repo/pull-commits";
import { PullConversation } from "@/components/repo/pull-conversation";
import { PullFiles } from "@/components/repo/pull-files";
import { PullMergePanel } from "@/components/repo/pull-merge-panel";
import { PullReviews } from "@/components/repo/pull-reviews";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import { apiClient } from "@/lib/api-client";
import { fetchRepoGet } from "@/lib/ssr-repo";
import { useChromeAccountState } from "@/lib/use-chrome-account";

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

export const Route = createFileRoute("/$owner/$repo/pull/$n")({
  component: PullDetailPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Pull #${(params as { n: string }).n} · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
  loader: async ({ params }) => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const repoName = String((params as { repo?: string }).repo ?? "");
    const n = Number((params as { n?: string }).n ?? 0);
    const got = await fetchRepoGet({ data: { owner, name: repoName } });
    if (!got.ok) {
      if (got.error.code === "repo.not_found") return { kind: "not_found" as const };
      return { kind: "error" as const, message: got.error.message || NETWORK_ERROR };
    }
    const pull = await apiClient.pull.get({ owner, name: repoName, number: n });
    if (!pull.ok) {
      if (pull.error.code === "repo.not_found" || pull.error.code === "pull.not_found") {
        return { kind: "not_found" as const };
      }
      return { kind: "error" as const, message: pull.error.message || NETWORK_ERROR };
    }
    return { kind: "ready" as const, repo: got.data, pull: pull.data };
  },
});

export function PullDetailPage() @{
  const params = useParams({ strict: false }) as {
    owner?: string;
    repo?: string;
    n?: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const number = Number(params.n ?? 0);
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<"conversation" | "commits" | "files">("conversation");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const pullQ = useQuery({
    queryKey: ["pull", "get", owner, repoName, number] as const,
    queryFn: async () => {
      const res = await apiClient.pull.get({ owner, name: repoName, number });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: !!owner && !!repoName && number > 0,
    initialData: (() => {
      const d = Route.useLoaderData() as
        | { kind: "ready"; pull: import("@octanest/api-client").PullPublic }
        | { kind: string };
      return d.kind === "ready" ? d.pull : undefined;
    })(),
  });

  const loader = Route.useLoaderData() as
    | { kind: "not_found" }
    | { kind: "error"; message: string }
    | {
        kind: "ready";
        repo: { can_write?: boolean };
        pull: import("@octanest/api-client").PullPublic;
      };

  const account = useChromeAccountState();
  const pull = pullQ.data;
  const canWrite = loader.kind === "ready" && !!loader.repo.can_write;
  const isAuthor = !!pull && !!account.user && account.user.id === (pull.author_id as string);

  async function closePull() {
    setBusy(true);
    setError(null);
    const res = await apiClient.pull.close({ owner, name: repoName, number });
    setBusy(false);
    if (!res.ok) {
      setError(res.error.message || "Close failed");
      return;
    }
    await queryClient.invalidateQueries({
      queryKey: ["pull", "get", owner, repoName, number],
    });
  }

  async function reopenPull() {
    setBusy(true);
    setError(null);
    const res = await apiClient.pull.reopen({ owner, name: repoName, number });
    setBusy(false);
    if (!res.ok) {
      setError(res.error.message || "Reopen failed");
      return;
    }
    await queryClient.invalidateQueries({
      queryKey: ["pull", "get", owner, repoName, number],
    });
  }

  @if (loader.kind === "not_found") {
    <RepoNotFound />;
  } @else {
    @if (loader.kind === "error") {
      <p className="text-destructive">{loader.message as string}</p>;
    } @else {
      @if (!pull) {
        <p className="text-muted-foreground">Loading…</p>;
      } @else {
        <div className="space-y-4">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 className="text-2xl font-semibold">
                {pull.title as string}{" "}
                <span className="text-muted-foreground">#{pull.number as number}</span>
              </h2>
              <p className="mt-1 text-sm text-muted-foreground">
                {pull.state as string} · {pull.author_username as string} wants to merge{" "}
                {pull.head_ref as string} into {pull.base_ref as string}
              </p>
            </div>
            @if (canWrite) {
              <div className="flex gap-2">
                @if (pull.state === "open") {
                  <button
                    type="button"
                    className="rounded-md border border-border px-3 py-1.5 text-sm"
                    disabled={busy}
                    onClick={() => void closePull()}
                  >
                    Close pull request
                  </button>;
                }
                @if (pull.state === "closed") {
                  <button
                    type="button"
                    className="rounded-md border border-border px-3 py-1.5 text-sm"
                    disabled={busy}
                    onClick={() => void reopenPull()}
                  >
                    Reopen pull request
                  </button>;
                }
              </div>;
            }
          </div>
          @if (error) {
            <p className="text-sm text-destructive">{error as string}</p>;
          }
          <PullMergePanel
            owner={owner}
            repoName={repoName}
            number={number}
            canWrite={canWrite}
            pullState={(pull.state as string) ?? "open"}
          />
          <div className="flex gap-3 border-b border-border" role="tablist">
            <button
              type="button"
              className={
                (tab === "conversation"
                  ? "border-b-2 border-primary pb-2 text-sm font-semibold"
                  : "pb-2 text-sm text-muted-foreground") as string
              }
              onClick={() => setTab("conversation")}
            >
              Conversation
            </button>
            <button
              type="button"
              className={
                (tab === "commits"
                  ? "border-b-2 border-primary pb-2 text-sm font-semibold"
                  : "pb-2 text-sm text-muted-foreground") as string
              }
              onClick={() => setTab("commits")}
            >
              Commits
            </button>
            <button
              type="button"
              className={
                (tab === "files"
                  ? "border-b-2 border-primary pb-2 text-sm font-semibold"
                  : "pb-2 text-sm text-muted-foreground") as string
              }
              onClick={() => setTab("files")}
            >
              Files changed
            </button>
          </div>
          @if (tab === "conversation") {
            <div className="space-y-6">
              <PullConversation
                owner={owner}
                repoName={repoName}
                number={number}
                canWrite={canWrite}
                body={(pull.body || "") as string}
              />
              <PullReviews
                owner={owner}
                repoName={repoName}
                number={number}
                canWrite={canWrite}
                isAuthor={isAuthor}
              />
            </div>;
          }
          @if (tab === "commits") {
            <PullCommits owner={owner} repoName={repoName} number={number} />;
          }
          @if (tab === "files") {
            <PullFiles
              owner={owner}
              repoName={repoName}
              number={number}
              canWrite={canWrite}
            />;
          }
        </div>;
      }
    }
  }
}
