import type { GpgKeyListItem } from "@oxidean/api-client";
import { gpgKeyListQueryOptions } from "@oxidean/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { KeyRound } from "@octanejs/lucide";
import { GpgKeyRevokeDialog } from "@/components/settings/gpg-key-revoke-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { List, ListRow } from "@/components/ui/list";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { formatRelativeUpdated } from "@/lib/relative-time";

const LIST_ERROR = "Couldn't load GPG keys. Check your connection and try again.";

function formatRelativeShort(iso: string): string {
  return formatRelativeUpdated(iso).replace(/^Updated\s+/i, "");
}

type GpgKeyListProps = {
  enabled?: boolean;
  initialData?: GpgKeyListItem[];
};

export function GpgKeyList({ enabled = true, initialData }: GpgKeyListProps) @{
  const queryClient = useQueryClient();
  const list = useQuery({
    ...gpgKeyListQueryOptions(apiClient),
    enabled,
    ...(initialData !== undefined ? { initialData } : {}),
  });
  const [revokeTarget, setRevokeTarget] = useState<GpgKeyListItem | null>(null);

  <>
    @if (list.isPending && initialData === undefined) {
      <div
        className="mt-8 flex flex-col gap-3"
        aria-busy="true"
        aria-label="Loading GPG keys"
      >
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-24 w-full" />
      </div>;
    } @else {
      @if (list.isError) {
        <div className="mt-8 flex flex-col gap-3">
          <p role="alert" className="text-[16px] text-destructive">
            {LIST_ERROR}
          </p>
          <Button
            type="button"
            variant="outline"
            className="w-fit"
            onClick={() => void list.refetch()}
          >
            Retry
          </Button>
        </div>;
      } @else {
        @if (!list.data || list.data.length === 0) {
          <EmptyState
            className="mt-8"
            icon={<KeyRound aria-hidden size={22} />}
            title="No GPG keys"
            description="Add an OpenPGP public key to verify GPG-signed commits."
          />;
        } @else {
          <List className="mt-8">
            @for (const item of list.data; key item.id) {
              <ListRow className="items-start justify-between gap-3 p-4">
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p
                      className="truncate text-[14px] font-semibold text-foreground"
                      title={item.title as string}
                    >
                      {item.title as string}
                    </p>
                    <Badge variant="outline">{item.key_id as string}</Badge>
                  </div>
                  <p className="mt-1 font-mono text-[12px] text-muted-foreground">
                    {item.fingerprint as string}
                  </p>
                  @if (item.uid_emails.length > 0) {
                    <p className="mt-2 text-[13px] text-muted-foreground">
                      {item.uid_emails.join(", ") as string}
                    </p>;
                  }
                  <p className="mt-2 text-[13px] text-muted-foreground">
                    {("Created · " + formatRelativeShort(item.created_at)) as string}
                  </p>
                </div>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="shrink-0 border-destructive/50 text-destructive hover:bg-destructive/10 hover:text-destructive"
                  onClick={() => setRevokeTarget(item)}
                >
                  Revoke
                </Button>
              </ListRow>;
            }
          </List>;
        }
      }
    }

    <GpgKeyRevokeDialog
      item={revokeTarget}
      open={revokeTarget !== null}
      onOpenChange={(open) => {
        if (!open) setRevokeTarget(null);
      }}
      onRevoked={() => {
        setRevokeTarget(null);
        void queryClient.invalidateQueries({ queryKey: ["gpgKey", "list"] });
      }}
    />
  </>;
}
