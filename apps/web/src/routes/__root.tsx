import {
  Body,
  Head,
  HeadContent,
  Html,
  Outlet,
  Scripts,
  createRootRoute,
} from "@octanejs/tanstack-router";
import { SiteFooter, SiteHeader } from "@/components/chrome";
import "@/styles.css";

export const Route = createRootRoute({
  component: RootComponent,
  shellComponent: RootShell,
  head: () => ({
    meta: [{ title: "Octanest" }],
  }),
});

function RootShell({ children }: { children?: unknown }) {
  return (
    <Html lang="en">
      <Head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <HeadContent />
      </Head>
      <Body>
        {children}
        <Scripts />
      </Body>
    </Html>
  );
}

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
