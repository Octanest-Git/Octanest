import { Outlet, createRootRoute } from "@octanejs/tanstack-router";
import { SiteFooter, SiteHeader } from "@/components/chrome";
import "@/styles.css";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  return (
    <div className="flex min-h-screen flex-col">
      <SiteHeader />
      <main className="flex-1">
        <Outlet />
      </main>
      <SiteFooter />
    </div>
  );
}
