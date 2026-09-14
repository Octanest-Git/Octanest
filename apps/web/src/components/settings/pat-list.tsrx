import type { PatListItem } from "@octanest/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { PatRevokeDialog } from "@/components/settings/pat-revoke-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { formatRelativeUpdated } from "@/lib/relative-time";
import { patListQueryOptions } from "@octanest/api-client";

const LIST_ERROR =
  "Couldn't load tokens. Check your connection and try again.";

function formatRelativeShort(iso: string): string {
  return formatRelativeUpdated(iso).replace(/^Updated\s+/i, "");
}

function formatExpiry(item: PatListItem, nowMs = Date.now()): string {
  if (!item.expires_at) return "No expiration";
  const ms = Date.parse(item.expires_at);
  if (Number.isNaN(ms)) return "Invalid expiration";
  return new Date(ms).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function isExpired(item: PatListItem, nowMs = Date.now()): boolean {
  if (!item.expires_at) return false;
  const ms = Date.parse(item.expires_at);
  if (Number.isNaN(ms)) return true;
  return ms < nowMs;
}

function scopeSummary(item: PatListItem): string {
  if (item.kind === "classic") {
    return "Full control of private repositories";
  }
  const contents =
    item.contents === "write" ? "Read and write" : "Read-only";
  const access =
    item.repo_access === "selected"
      ? "Only select repositories"
      : "All repositories";
  return `${contents} · ${access}`;
}

function lastUsedLabel(item: PatListItem): string {
  if (!item.last_used_at) return "Never";
  const when = formatRelativeShort(item.last_used_at);
  if (item.last_used_ip) return `${when} from ${item.last_used_ip}`;
  return when;
}

type PatListProps = {
  /** When false, list query is still enabled for viewing existing tokens. */
  enabled?: boolean;
};

/** PAT list rows + empty/loading/error (D-14 / D-09). */
export function PatList({ enabled = true }: PatListProps) @{
  const queryClient = useQueryClient();
  const list = useQuery({
    ...patListQueryOptions(apiClient),
    enabled,
  });
  const [revokeTarget, setRevokeTarget] = useState<PatListItem | null>(null);

  <>
  @if (list.isPending) {
    <div className="mt-8 flex flex-col gap-3" aria-busy="true" aria-label="Loading tokens">
      <Skeleton className="h-24 w-full" />
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
            No personal access tokens
          </h2>
          <p className="text-[16px] text-muted-foreground">
            Create a token to clone, fetch, and push over HTTPS.
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
                      title={item.name as string}
                    >
                      {item.name as string}
                    </p>
                    <Badge variant="outline">
                      {(item.kind === "fine_grained" ? "Fine-grained" : "Classic") as string}
                    </Badge>
                    @if (isExpired(item)) {
                      <Badge
                        variant="outline"
                        className="border-destructive/40 text-destructive"
                      >
                        Expired
                      </Badge>
                    }
                  </div>
                  <p className="mt-1 text-[14px] text-muted-foreground">
                    {scopeSummary(item) as string}
                  </p>
                  <p className="mt-1 font-mono text-[12px] text-muted-foreground">
                    {(item.token_prefix + "…") as string}
                  </p>
                  <p className="mt-2 text-[13px] text-muted-foreground">
                    {("Expires · " + formatExpiry(item)) as string}
                  </p>
                  <p className="text-[13px] text-muted-foreground">
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
                  Revoke token
                </Button>
              </div>
            </li>
          }
        </ul>
      }
    }
  }

  <PatRevokeDialog
    item={revokeTarget}
    open={revokeTarget !== null}
    onOpenChange={(open) => {
      if (!open) setRevokeTarget(null);
    }}
    onRevoked={() => {
      setRevokeTarget(null);
      void queryClient.invalidateQueries({ queryKey: ["pat", "list"] });
    }}
  />
  </>
}
