import { queryOptions, type QueryClient } from "@octanejs/tanstack-query";
import type {
  NotificationListResponse,
  NotificationPublic,
  NotificationUnreadCountResponse,
} from "@oxidean/api-client";
import { apiClient } from "@/lib/api-client";

export const notificationUnreadCountQueryKey = ["notification", "unreadCount"] as const;

export function notificationListQueryKey(filter: "unread" | "all", offset = 0) {
  return ["notification", "list", filter, offset] as const;
}

/** Soft unread badge — unauthenticated → 0 (chrome must not throw). */
export function notificationUnreadCountQueryOptions() {
  return queryOptions({
    queryKey: notificationUnreadCountQueryKey,
    queryFn: async (): Promise<number> => {
      const res = await apiClient.notification.unreadCount({});
      if (!res.ok) {
        if (res.error.code === "auth.unauthenticated" || res.error.code === "auth.setup_required") {
          return 0;
        }
        throw new Error(`${res.error.code}: ${res.error.message}`);
      }
      return (res.data as NotificationUnreadCountResponse).count;
    },
    retry: false,
    staleTime: 15_000,
    refetchInterval: 30_000,
  });
}

export function notificationListQueryOptions(filter: "unread" | "all", offset = 0, limit = 30) {
  return queryOptions({
    queryKey: notificationListQueryKey(filter, offset),
    queryFn: async (): Promise<NotificationListResponse> => {
      const res = await apiClient.notification.list({ filter, offset, limit });
      if (!res.ok) {
        throw new Error(`${res.error.code}: ${res.error.message}`);
      }
      return res.data;
    },
    retry: false,
    staleTime: 10_000,
  });
}

export function invalidateNotificationQueries(qc: QueryClient) {
  void qc.invalidateQueries({ queryKey: ["notification"] });
}

export function subjectHref(n: NotificationPublic): string {
  if (n.subject_kind === "pull_request") {
    return `/${n.owner}/${n.repo}/pull/${n.subject_number}`;
  }
  return `/${n.owner}/${n.repo}/issues/${n.subject_number}`;
}

export function reasonLabel(reason: string): string {
  switch (reason) {
    case "issue_opened":
      return "opened an issue";
    case "issue_closed":
      return "closed an issue";
    case "issue_reopened":
      return "reopened an issue";
    case "issue_comment":
      return "commented on";
    case "issue_assigned":
      return "assigned you";
    case "issue_unassigned":
      return "unassigned you";
    case "issue_mention":
      return "mentioned you";
    case "pr_opened":
      return "opened a pull request";
    case "pr_closed":
      return "closed a pull request";
    case "pr_reopened":
      return "reopened a pull request";
    case "pr_merged":
      return "merged a pull request";
    case "pr_review":
      return "reviewed";
    case "pr_comment":
      return "commented on";
    case "pr_review_requested":
      return "requested your review";
    case "pr_mention":
      return "mentioned you";
    default:
      return reason.replace(/_/g, " ");
  }
}
