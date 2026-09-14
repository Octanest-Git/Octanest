import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for `/{owner}/{repo}/releases/*`.
 * List lives on the index route so `$tag` / `new` can render.
 */
export const Route = createFileRoute("/$owner/$repo/releases")({
  component: ReleasesLayout,
});

function ReleasesLayout() @{
  <Outlet />
}
