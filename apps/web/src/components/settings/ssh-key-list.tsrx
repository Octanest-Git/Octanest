import type { SshKeyListItem } from "@octanest/api-client";
import { sshKeyListQueryOptions } from "@octanest/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { SshKeyRevokeDialog } from "@/components/settings/ssh-key-revoke-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { formatRelativeUpdated } from "@/lib/relative-time";

const LIST_ERROR =
  "Couldn't load SSH keys. Check your connection and try again.";

function formatRelativeShort(iso: string): string {
  return formatRelativeUpdated(iso).replace(/^Updated\s+/i, "");
}

function lastUsedLabel(item: SshKeyListItem): string {
  if (!item.last_used_at) return "Never";
  const when = formatRelativeShort(item.last_used_at);
  if (item.last_used_ip) return `${when} from ${item.last_used_ip}`;
  return when;
}

type SshKeyListProps = {
  enabled?: boolean;
  /** SSR / loader seed — skips the pending skeleton when provided. */
  initialData?: SshKeyListItem[];
};

/** SSH key list rows + empty/loading/error (D-SSH-05 / D-SSH-06). */
export function SshKeyList({ enabled = true, initialData }: SshKeyListProps) @{
  const queryClient = useQueryClient();
  const list = useQuery({
    ...sshKeyListQueryOptions(apiClient),
    enabled,
    ...(initialData !== undefined ? { initialData } : {}),
  });
  const [revokeTarget, setRevokeTarget] = useState<SshKeyListItem | null>(null);

  <>
  @if (list.isPending && initialData === undefined) {
    <div className="mt-8 flex flex-col gap-3" aria-busy="true" aria-label="Loading SSH keys">
      <Skeleton className="h-24 w-full" />
      <Skeleton className="h-24 w-full" />
    </div>
  } @else {
    @if (list.isError) {
      <div className="mt-8 flex flex-col gap-3">
        <p role="alert" className="text-[16px] text-destructive">
          {LIST_ERROR}
        </p>
        <Button
          type="button"
          variant="ghost"
          className="w-fit"
          onClick={() => void list.refetch()}
        >
          Retry
        </Button>
      </div>
    } @else {
      @if (!list.data || list.data.length === 0) {
        <div className="mt-10 flex flex-col items-start gap-2">
          <h2 className="font-[family-name:var(--font-display)] text-[20px] font-semibold leading-[1.2] text-foreground">
            No SSH keys
          </h2>
          <p className="text-[16px] text-muted-foreground">
            Add a public key to clone, fetch, and push over SSH.
          </p>
        </div>
      } @else {
        <ul className="mt-8 flex list-none flex-col gap-3 p-0">
          @for (const item of list.data; key item.id) {
            <li className="rounded-md border border-border bg-card p-4">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p
                      className="truncate text-[14px] font-semibold text-foreground"
                      title={item.title as string}
                    >
                      {item.title as string}
                    </p>
                    <Badge variant="outline">
                      {item.key_type as string}
                    </Badge>
                  </div>
                  <p className="mt-1 font-mono text-[12px] text-muted-foreground">
                    {item.fingerprint as string}
                  </p>
                  <p className="mt-2 text-[13px] text-muted-foreground">
                    {("Last used · " + lastUsedLabel(item)) as string}
                  </p>
                  <p className="text-[13px] text-muted-foreground">
                    {("Created · " + formatRelativeShort(item.created_at)) as string}
                  </p>
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  className="h-11 shrink-0 border border-destructive text-destructive hover:bg-destructive/10"
                  onClick={() => setRevokeTarget(item)}
                >
                  Revoke
                </Button>
              </div>
            </li>
          }
        </ul>
      }
    }
  }

  <SshKeyRevokeDialog
    item={revokeTarget}
    open={revokeTarget !== null}
    onOpenChange={(open) => {
      if (!open) setRevokeTarget(null);
    }}
    onRevoked={() => {
      setRevokeTarget(null);
      void queryClient.invalidateQueries({ queryKey: ["sshKey", "list"] });
    }}
  />
  </>
}
