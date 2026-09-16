import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/** Layout for `/{owner}/{repo}/actions/*` — no RepoChrome remount (D-QH-01). */
export const Route = createFileRoute("/$owner/$repo/actions")({
  component: ActionsLayout,
});

function ActionsLayout() @{
  <Outlet />;
}
