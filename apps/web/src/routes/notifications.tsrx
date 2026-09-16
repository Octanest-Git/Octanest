import { createFileRoute, redirect } from "@octanejs/tanstack-router";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import type { NotificationPublic } from "@octanest/api-client";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";
import {
  invalidateNotificationQueries,
  notificationListQueryOptions,
  notificationUnreadCountQueryOptions,
  reasonLabel,
  subjectHref,
} from "@/lib/notification-queries";
import { fetchSessionMe } from "@/lib/ssr-auth";
import { cn } from "@/lib/utils";

export const Route = createFileRoute("/notifications")({
  component: NotificationsPage,
  head: () => ({ meta: [{ title: "Notifications · Octanest" }] }),
  beforeLoad: async () => {
    try {
      const me = await fetchSessionMe();
      if (!me.ok) {
        throw redirect({ href: "/login?returnTo=/notifications" });
      }
    } catch (e) {
      if (e && typeof e === "object" && "href" in e) throw e;
      throw redirect({ href: "/login?returnTo=/notifications" });
    }
  },
});

/** Exported for integration tests (NOTF-02 / D-09 / D-12). */
export function NotificationsPage() @{
  const qc = useQueryClient();
  const [filter, setFilter] = useState<"unread" | "all">("unread");
  const [offset, setOffset] = useState(0);
  const listQ = useQuery(notificationListQueryOptions(filter, offset));
  const unreadQ = useQuery(notificationUnreadCountQueryOptions());
  const rows: NotificationPublic[] = listQ.data?.notifications ?? [];
  const total = listQ.data?.total ?? 0;
  const unreadCount = unreadQ.data ?? 0;
  const limit = 30;

  async function markAll() {
    const res = await apiClient.notification.markAllRead({});
    if (res.ok) {
      invalidateNotificationQueries(qc);
    }
  }

  async function activateRow(n: NotificationPublic) {
    if (!n.read_at) {
      await apiClient.notification.markRead({ ids: [n.id] });
      invalidateNotificationQueries(qc);
    }
    window.location.assign(subjectHref(n));
  }

  <div className="mx-auto max-w-3xl px-4 py-10">
    <div className="flex flex-wrap items-end justify-between gap-4">
      <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold tracking-tight">
        Notifications
      </h1>
      @if (unreadCount > 0) {
        <Button type="button" variant="secondary" onClick={() => void markAll()}>
          Mark all as read
        </Button>;
      }
    </div>

    <div
      role="tablist"
      aria-label="Notification filters"
      className="mt-6 inline-flex rounded-md border border-border p-1"
    >
      <button
        type="button"
        role="tab"
        aria-selected={filter === "unread"}
        className={cn(
          "rounded px-3 py-1.5 text-[14px] font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring",
          filter === "unread" ? "bg-muted text-foreground" : "text-muted-foreground",
        )}
        onClick={() => {
          setFilter("unread");
          setOffset(0);
        }}
      >
        Unread
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={filter === "all"}
        className={cn(
          "rounded px-3 py-1.5 text-[14px] font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring",
          filter === "all" ? "bg-muted text-foreground" : "text-muted-foreground",
        )}
        onClick={() => {
          setFilter("all");
          setOffset(0);
        }}
      >
        All
      </button>
    </div>

    <div className="mt-6">
      @if (listQ.isPending) {
        <p className="text-[14px] text-muted-foreground">Loading notifications…</p>;
      } @else {
        @if (rows.length === 0) {
          <p className="text-[15px] text-muted-foreground">
            {filter === "unread"
              ? ("You're all caught up." as string)
              : ("No notifications yet." as string)}
          </p>;
        } @else {
          <ul className="divide-y divide-border border-y border-border">
            @for (const n of rows; key n.id) {
              <li>
                <button
                  type="button"
                  className={cn(
                    "flex w-full flex-col gap-0.5 px-1 py-3 text-left outline-none",
                    "hover:bg-muted/60 focus-visible:ring-2 focus-visible:ring-ring",
                    !n.read_at ? "font-medium" : "text-muted-foreground",
                  )}
                  aria-label={`${n.actor_username} ${reasonLabel(n.reason)} ${n.owner}/${n.repo} #${n.subject_number}`}
                  onClick={() => void activateRow(n)}
                >
                  <span className="text-[14px] text-foreground">
                    <span className="font-semibold">{n.actor_username as string}</span>{" "}
                    {reasonLabel(n.reason) as string}{" "}
                    <span className="text-foreground">
                      {n.owner as string}/{n.repo as string}#
                      {String(n.subject_number) as string}
                    </span>
                  </span>
                  <span className="text-[13px] text-muted-foreground">
                    {n.subject_title as string}
                    {" · "}
                    {n.created_at as string}
                  </span>
                </button>
              </li>;
            }
          </ul>;
          @if (offset + rows.length < total) {
            <div className="mt-4">
              <Button
                type="button"
                variant="secondary"
                onClick={() => setOffset(offset + limit)}
              >
                Older
              </Button>
            </div>;
          }
        }
      }
    </div>
  </div>;
}
