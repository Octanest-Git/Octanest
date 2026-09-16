import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/** Layout for `/{owner}/{repo}/pulls/*`. */
export const Route = createFileRoute("/$owner/$repo/pulls")({
  component: PullsLayout,
});

function PullsLayout() @{
  <Outlet />;
}
