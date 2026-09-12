import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for `/{owner}/{repo}` browse surfaces (D-14).
 * Code home lives in `$owner.$repo.index`; later plans add tree/blob/settings children.
 */
export const Route = createFileRoute("/$owner/$repo")({
  component: RepoLayout,
});

function RepoLayout() @{
  <Outlet />
}
