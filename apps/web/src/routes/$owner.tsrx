import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for `/{owner}/*` — must NOT require an organization.
 * User-owned repos (`/{owner}/{repo}`), packages, and org settings are children;
 * org overview lives on the index route (`$owner.index.tsrx`).
 */
export const Route = createFileRoute("/$owner")({
  component: OwnerLayout,
});

function OwnerLayout() @{
  <Outlet />;
}
