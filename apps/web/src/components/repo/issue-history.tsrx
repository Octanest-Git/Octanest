import { useQuery } from "@octanejs/tanstack-query";
import type { IssueRevisionPublic } from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";

export type IssueHistoryPanelProps = {
  owner: string;
  repoName: string;
  number: number;
};

/** Prior title/body revision trail (D-ISS-04). */
export function IssueHistoryPanel(props: IssueHistoryPanelProps) @{
  const historyQ = useQuery({
    queryKey: ["issue", "history", props.owner, props.repoName, props.number] as const,
    queryFn: async () => {
      const res = await apiClient.issue.history({
        owner: props.owner,
        name: props.repoName,
        number: props.number,
      });
      if (!res.ok) throw new Error(res.error.message || "Could not load history.");
      return res.data;
    },
    enabled: !!props.owner && !!props.repoName && props.number > 0,
  });

  const revisions: IssueRevisionPublic[] = historyQ.data?.revisions ?? [];
  const showList = !historyQ.isPending && !historyQ.isError && revisions.length > 0;
  const showEmpty =
    !historyQ.isPending && !historyQ.isError && revisions.length === 0;

  <section aria-labelledby="issue-history-heading" className="flex flex-col gap-3">
    <h2
      id="issue-history-heading"
      className="text-[16px] font-semibold text-foreground"
    >
      Edit history
    </h2>
    @if (historyQ.isPending) {
      <p className="text-[14px] text-muted-foreground">Loading history…</p>
    } @else {
      @if (historyQ.isError) {
        <p role="alert" className="text-[14px] text-destructive">
          Could not load edit history.
        </p>
      } @else {
        @if (showEmpty) {
          <p className="text-[14px] text-muted-foreground">No prior edits.</p>
        }
        @if (showList) {
          <ul className="flex flex-col gap-3">
            @for (const rev of revisions; key rev.id) {
              <li className="rounded-md border border-border px-3 py-2 text-[13px]">
                <p className="text-muted-foreground">
                  {`@${rev.editor_username} · ${rev.created_at}` as string}
                </p>
                <p className="mt-1 font-medium text-foreground">
                  {rev.title as string}
                </p>
                <pre className="mt-1 whitespace-pre-wrap font-sans text-muted-foreground">
                  {rev.body as string}
                </pre>
              </li>
            }
          </ul>
        }
      }
    }
  </section>
}
