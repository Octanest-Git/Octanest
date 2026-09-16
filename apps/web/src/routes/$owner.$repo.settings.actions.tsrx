import { createFileRoute, redirect } from "@octanejs/tanstack-router";

export const Route = createFileRoute("/$owner/$repo/settings/actions")({
  beforeLoad: ({ params }) => {
    const p = params as { owner: string; repo: string };
    throw redirect({
      to: "/$owner/$repo/settings",
      params: { owner: p.owner, repo: p.repo },
      hash: "actions",
    });
  },
});
