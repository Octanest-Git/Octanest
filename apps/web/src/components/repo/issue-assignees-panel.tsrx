import type { IssueAssigneePublic, IssuePublic } from "@oxidean/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useEffect, useState } from "octane";
import { AuthErrorBanner } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";

const NETWORK_ERROR = "Can't reach Oxidean. Check your connection and try again.";

type IssueAssigneesPanelProps = {
  owner: string;
  repoName: string;
  number: number;
  issue: IssuePublic;
  canWrite: boolean;
};

/** Issue sidebar assignees — Write+ multi-select from Read+ candidates (ISS-03 / D-ISS-06..08). */
export function IssueAssigneesPanel({
  owner,
  repoName,
  number,
  issue,
  canWrite,
}: IssueAssigneesPanelProps) @{
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<string[]>(() =>
    (issue.assignees ?? []).map((a) => a.user_id),
  );
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    setSelected((issue.assignees ?? []).map((a) => a.user_id));
  }, [issue.id, issue.updated_at]);

  const candidatesQ = useQuery({
    queryKey: ["issue", "assigneeCandidates", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.issue.assigneeCandidates({
        owner,
        name: repoName,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data.users;
    },
    enabled: !!owner && !!repoName && canWrite,
  });

  const current: IssueAssigneePublic[] = issue.assignees ?? [];
  const catalog: IssueAssigneePublic[] = candidatesQ.data ?? [];

  function toggle(userId: string) {
    setSelected((prev) =>
      prev.includes(userId) ? prev.filter((x) => x !== userId) : [...prev, userId],
    );
  }

  async function save() {
    setError("");
    setPending(true);
    try {
      const res = await apiClient.issue.assignees.set({
        owner,
        name: repoName,
        number,
        userIds: selected,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not update assignees.");
        setPending(false);
        return;
      }
      await queryClient.invalidateQueries({
        queryKey: ["issue", "get", owner, repoName, number],
      });
      await queryClient.invalidateQueries({
        queryKey: ["issue", "list", owner, repoName],
      });
      setPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  <section aria-labelledby="issue-assignees-heading">
    <h2 id="issue-assignees-heading" className="text-[14px] font-semibold text-foreground">
      Assignees
    </h2>
    @if (error) {
      <div className="mt-2">
        <AuthErrorBanner message={error} />
      </div>;
    }
    @if (canWrite) {
      <div className="mt-2 flex flex-col gap-2">
        @if (candidatesQ.isPending) {
          <p className="text-[13px] text-muted-foreground">Loading assignees…</p>;
        } @else {
          @if (catalog.length === 0) {
            <p className="text-[13px] text-muted-foreground">
              No eligible assignees for this repository.
            </p>;
          } @else {
            <ul className="flex flex-col gap-1.5">
              @for (const user of catalog; key user.user_id) {
                <li>
                  <label className="flex cursor-pointer items-center gap-2 text-[13px] text-foreground">
                    <input
                      type="checkbox"
                      checked={selected.includes(user.user_id)}
                      disabled={pending}
                      onChange={() => toggle(user.user_id)}
                    />
                    <span>{user.username as string}</span>
                    @if (
                      user.display_name &&
                      user.display_name !== user.username
                    ) {
                      <span className="text-muted-foreground">
                        ({user.display_name as string})
                      </span>;
                    }
                  </label>
                </li>;
              }
            </ul>;
          }
        }
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={pending || candidatesQ.isPending}
          onClick={save}
        >
          Save assignees
        </Button>
      </div>;
    } @else {
      <div className="mt-2">
        @if (current.length === 0) {
          <p className="text-[13px] text-muted-foreground">None yet</p>;
        } @else {
          <ul className="flex flex-wrap gap-1.5">
            @for (const user of current; key user.user_id) {
              <li className="inline-flex items-center rounded-md border border-border px-2 py-0.5 text-[12px] text-foreground">
                {user.username as string}
              </li>;
            }
          </ul>;
        }
      </div>;
    }
  </section>;
}
