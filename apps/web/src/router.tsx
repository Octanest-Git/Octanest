import { createRouter } from "@octanejs/tanstack-router";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
  return createRouter({
    routeTree,
    scrollRestoration: true,
  });
}

declare module "@octanejs/tanstack-router" {
  interface Register {
    router: ReturnType<typeof getRouter>;
  }
}
