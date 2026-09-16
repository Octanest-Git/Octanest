import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for `/settings/tokens/new/*`.
 * Classic create lives on the index so fine-grained can render.
 */
export const Route = createFileRoute("/settings/tokens/new")({
  component: TokensNewLayout,
  head: () => ({ meta: [{ title: "New token · Octanest" }] }),
});

function TokensNewLayout() @{
  <Outlet />;
}
