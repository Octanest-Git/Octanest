import type { FileRouteTypes } from "@/routeTree.gen";

/** Union of typed `Link` / `navigate({ to })` destinations from the route tree. */
export type AppTo = FileRouteTypes["to"];

/** Union of full pathnames generated for this app. */
export type AppPath = FileRouteTypes["fullPaths"];
