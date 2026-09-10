import { createRouter } from "@octanejs/tanstack-router";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
  return createRouter({
    routeTree,
    // Typed route tree is registered below — Link `to` / params infer from it.
    defaultPreload: "intent",
    defaultPreloadDelay: 50,
    // Smooth cross-route transitions via the View Transitions API (Octane adapter
    // commits match updates inside startViewTransition).
    defaultViewTransition: true,
    // Restore scroll on route changes; same-document hash jumps stay native.
    scrollRestoration: true,
  });
}

declare module "@octanejs/tanstack-router" {
  interface Register {
    router: ReturnType<typeof getRouter>;
  }
}
