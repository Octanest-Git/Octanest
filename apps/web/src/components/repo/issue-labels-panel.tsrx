import type { IssuePublic, LabelPublic } from "@octanest/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useEffect, useState } from "octane";
import { AuthErrorBanner } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

type IssueLabelsPanelProps = {
  owner: string;
  repoName: string;
  number: number;
  issue: IssuePublic;
  canWrite: boolean;
};

function swatchStyle(color: string): { backgroundColor: string } {
  const hex = color.startsWith("#") ? color : `#${color}`;
  return { backgroundColor: hex };
}

/** Issue sidebar labels — Write+ assign from effective set (ISS-03 / D-ISS-07). */
export function IssueLabelsPanel({
  owner,
  repoName,
  number,
  issue,
  canWrite,
}: IssueLabelsPanelProps) @{
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<string[]>(() =>
    (issue.labels ?? []).map((l) => l.id),
  );
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    setSelected((issue.labels ?? []).map((l) => l.id));
  }, [issue.id, issue.updated_at]);

  const catalogQ = useQuery({
    queryKey: ["label", "listForRepo", owner, repoName] as const,
    queryFn: async () => {
      const res = await apiClient.label.listForRepo({
        owner,
        name: repoName,
      });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data.labels;
    },
    enabled: !!owner && !!repoName && canWrite,
  });

  const current: LabelPublic[] = issue.labels ?? [];
  const catalog: LabelPublic[] = catalogQ.data ?? [];

  function toggle(id: string) {
    setSelected((prev) =>
      prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
    );
  }

  async function save() {
    setError("");
    setPending(true);
    try {
      const res = await apiClient.issue.labels.set({
        owner,
        name: repoName,
        number,
        labelIds: selected,
      });
      if (!res.ok) {
        setError(res.error.message || "Could not update labels.");
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

  <section aria-labelledby="issue-labels-heading">
    <h2
      id="issue-labels-heading"
      className="text-[14px] font-semibold text-foreground"
    >
      Labels
    </h2>
    @if (error) {
      <div className="mt-2">
        <AuthErrorBanner message={error} />
      </div>
    }
    @if (canWrite) {
      <div className="mt-2 flex flex-col gap-2">
        @if (catalogQ.isPending) {
          <p className="text-[13px] text-muted-foreground">Loading labels…</p>
        } @else {
          @if (catalog.length === 0) {
            <p className="text-[13px] text-muted-foreground">
              No labels in this repository yet.
            </p>
          } @else {
            <ul className="flex flex-col gap-1.5">
              @for (const lab of catalog; key lab.id) {
                <li>
                  <label className="flex cursor-pointer items-center gap-2 text-[13px] text-foreground">
                    <input
                      type="checkbox"
                      checked={selected.includes(lab.id)}
                      disabled={pending}
                      onChange={() => toggle(lab.id)}
                    />
                    <span
                      className="inline-block size-3 shrink-0 rounded-sm border border-border"
                      style={swatchStyle(lab.color)}
                      aria-hidden
                    />
                    <span>{lab.name as string}</span>
                  </label>
                </li>
              }
            </ul>
          }
        }
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={pending || catalogQ.isPending}
          onClick={save}
        >
          Save labels
        </Button>
      </div>
    } @else {
      <div className="mt-2">
        @if (current.length === 0) {
          <p className="text-[13px] text-muted-foreground">None yet</p>
        } @else {
          <ul className="flex flex-wrap gap-1.5">
            @for (const lab of current; key lab.id) {
              <li
                className="inline-flex items-center gap-1.5 rounded-md border border-border px-2 py-0.5 text-[12px] text-foreground"
              >
                <span
                  className="inline-block size-2.5 rounded-sm"
                  style={swatchStyle(lab.color)}
                  aria-hidden
                />
                {lab.name as string}
              </li>
            }
          </ul>
        }
      </div>
    }
  </section>
}
