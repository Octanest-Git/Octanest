import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for `/settings/tokens/*`.
 * List lives on the index route so `/new` and fine-grained can render.
 */
export const Route = createFileRoute("/settings/tokens")({
  component: TokensLayout,
  head: () => ({ meta: [{ title: "Personal access tokens · Oxidean" }] }),
});

function TokensLayout() @{
  <Outlet />;
}
