import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for `/{owner}/{repo}/issues/*`.
 * List lives on the index route so `$n` / `new` / `labels` can render.
 */
export const Route = createFileRoute("/$owner/$repo/issues")({
  component: IssuesLayout,
});

function IssuesLayout() @{
  <Outlet />
}
