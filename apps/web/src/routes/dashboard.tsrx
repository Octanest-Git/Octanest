import { createFileRoute, notFound } from "@octanejs/tanstack-router";

/**
 * Legacy path — not a public page (D-19). Direct hits hard-404.
 * Shared root SSR gate already handles needs_setup / must_change for this path.
 */
export const Route = createFileRoute("/dashboard")({
  beforeLoad: () => {
    throw notFound();
  },
  component: DashboardGone,
  head: () => ({ meta: [{ title: "Oxidean" }] }),
});

/** Unreachable when beforeLoad throws notFound — kept so the route tree stays valid. */
function DashboardGone() {
  return null;
}
