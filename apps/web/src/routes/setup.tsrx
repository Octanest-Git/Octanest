import { Outlet, createFileRoute } from "@octanejs/tanstack-router";

/**
 * Layout for /setup and /setup/credentials — child routes render via Outlet.
 * Wizard lives in setup.index; forced-change in setup.credentials.
 */
export const Route = createFileRoute("/setup")({
  component: SetupLayout,
});

function SetupLayout() @{
  <Outlet />
}

export { SetupPage } from "./setup.index";
