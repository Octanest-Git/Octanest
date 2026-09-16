import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import type { MergeMethod, RepoMergeSettings } from "@octanest/api-client";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";

export type PullMergePanelProps = {
  owner: string;
  repoName: string;
  number: number;
  canWrite: boolean;
  pullState: string;
};

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

export function PullMergePanel(props: PullMergePanelProps) @{
  const queryClient = useQueryClient();
  const [method, setMethod] = useState<MergeMethod>("merge");
  const [deleteBranch, setDeleteBranch] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const settingsQ = useQuery({
    queryKey: ["mergeSettings", props.owner, props.repoName] as const,
    queryFn: async () => {
      const res = await apiClient.mergeSettings.get({
        owner: props.owner,
        name: props.repoName,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data;
    },
    enabled: props.canWrite && props.pullState === "open",
  });

  const settings: RepoMergeSettings | undefined = settingsQ.data;

  async function merge() {
    setBusy(true);
    setError(null);
    const res = await apiClient.pull.merge({
      owner: props.owner,
      name: props.repoName,
      number: props.number,
      method,
      delete_branch: deleteBranch,
    });
    setBusy(false);
    if (!res.ok) {
      setError(res.error.message || "Merge failed");
      return;
    }
    await queryClient.invalidateQueries({
      queryKey: ["pull", "get", props.owner, props.repoName, props.number],
    });
  }

  @if (!props.canWrite || props.pullState !== "open") {
    <div />;
  } @else {
    <div className="space-y-3 rounded-md border border-border p-3" data-testid="pull-merge-panel">
      <h3 className="text-sm font-semibold">Merge pull request</h3>
      @if (settingsQ.isPending) {
        <p className="text-sm text-muted-foreground">Loading merge options…</p>;
      } @else {
        <div className="flex flex-wrap gap-2" role="group" aria-label="Merge method">
          @if (!settings || settings.allow_merge_commit) {
            <button
              type="button"
              className={
                (method === "merge"
                  ? "rounded-md bg-muted px-2 py-1 text-sm font-medium"
                  : "rounded-md px-2 py-1 text-sm text-muted-foreground") as string
              }
              onClick={() => setMethod("merge")}
            >
              Create a merge commit
            </button>;
          }
          @if (!settings || settings.allow_squash_merge) {
            <button
              type="button"
              className={
                (method === "squash"
                  ? "rounded-md bg-muted px-2 py-1 text-sm font-medium"
                  : "rounded-md px-2 py-1 text-sm text-muted-foreground") as string
              }
              onClick={() => setMethod("squash")}
            >
              Squash and merge
            </button>;
          }
          @if (!settings || settings.allow_rebase_merge) {
            <button
              type="button"
              className={
                (method === "rebase"
                  ? "rounded-md bg-muted px-2 py-1 text-sm font-medium"
                  : "rounded-md px-2 py-1 text-sm text-muted-foreground") as string
              }
              onClick={() => setMethod("rebase")}
            >
              Rebase and merge
            </button>;
          }
        </div>;
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={deleteBranch}
            onChange={(e) => setDeleteBranch((e.target as HTMLInputElement).checked)}
          />
          Delete head branch after merge
        </label>;
        <Button type="button" disabled={busy} onClick={() => void merge()}>
          Merge pull request
        </Button>;
      }
      @if (error) {
        <p className="text-sm text-destructive">{error as string}</p>;
      }
    </div>;
  }
}
