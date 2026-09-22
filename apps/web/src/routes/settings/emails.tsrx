import { createFileRoute, redirect } from "@octanejs/tanstack-router";

/** Emails live on Account (`/settings/profile`); keep this path as a redirect. */
export const Route = createFileRoute("/settings/emails")({
  beforeLoad: () => {
    throw redirect({ href: "/settings/profile" });
  },
});
