import {
  Body,
  Head,
  HeadContent,
  Html,
  Outlet,
  Scripts,
  createRootRoute,
  redirect,
  useLoaderData,
} from "@octanejs/tanstack-router";
import {
  dehydrate,
  HydrationBoundary,
  QueryClientProvider,
  type DehydratedState,
} from "@octanejs/tanstack-query";
import type {
  BootstrapStatus,
  ProviderConfigPublic,
  UserPublic,
} from "@octanest/api-client";
import { SiteFooter, SiteHeader } from "@/components/chrome";
import { AppToaster } from "@/components/ui/toaster";
import { VerifyBanner } from "@/components/verify-banner";
import { createAppQueryClient, getQueryClient } from "@/lib/query-client";
import {
  authBootstrapQueryKey,
  authProviderConfigQueryKey,
  authMeQueryKey,
} from "@/lib/session-queries";
import {
  fetchBootstrapStatus,
  fetchProviderConfig,
  fetchSessionMe,
  resolveAppAccessRedirect,
} from "@/lib/ssr-auth";
import { THEME_BOOT_SCRIPT, FONT_WARM_SCRIPT } from "@/lib/theme";
import "@/styles.css";

// Static literal, no interpolation (T-03-19) — registers the hand-authored
// assets-only service worker (see apps/web/public/sw.js, 03-05-SUMMARY.md).
const SW_REGISTER_SCRIPT =
  '(function(){if("serviceWorker" in navigator){window.addEventListener("load",' +
  'function(){navigator.serviceWorker.register("/sw.js").catch(function(){});});}})();';

const PROVIDER_CONFIG_LOCKED: ProviderConfigPublic = {
  mode: "local",
  allow_signup: false,
};

export type RootLoaderData = {
  dehydratedState: DehydratedState;
};

/**
 * SSR gate + seed auth chrome queries (me / bootstrap / providerConfig) so
 * SiteHeader paints without a client-only skeleton flash.
 */
export const Route = createRootRoute({
  component: RootComponent,
  shellComponent: RootShell,
  head: () => ({
    meta: [{ title: "Octanest" }],
  }),
  loader: async ({ location }): Promise<RootLoaderData> => {
    const qc = createAppQueryClient();

    let needsSetup = false;
    let boot: BootstrapStatus = { needs_setup: false };
    try {
      const status = await fetchBootstrapStatus();
      if (status.ok) {
        boot = status.data;
        needsSetup = Boolean(status.data.needs_setup);
      }
    } catch {
      /* API unreachable — do not lock the tree */
    }
    qc.setQueryData(authBootstrapQueryKey, boot);

    let me: UserPublic | null = null;
    let mustChangeCredentials = false;
    if (!needsSetup) {
      try {
        const meRes = await fetchSessionMe();
        if (meRes.ok) {
          me = meRes.data;
          mustChangeCredentials = Boolean(me.must_change_credentials);
        }
      } catch {
        /* anonymous / network */
      }
    }
    qc.setQueryData(authMeQueryKey, me);

    let cfg: ProviderConfigPublic = PROVIDER_CONFIG_LOCKED;
    try {
      const cfgRes = await fetchProviderConfig();
      if (cfgRes.ok) {
        cfg = cfgRes.data;
      }
    } catch {
      /* fail closed */
    }
    qc.setQueryData(authProviderConfigQueryKey, cfg);

    const target = resolveAppAccessRedirect({
      pathname: location.pathname,
      needsSetup,
      mustChangeCredentials,
    });
    if (target) {
      throw redirect({ href: target });
    }

    return { dehydratedState: dehydrate(qc) };
  },
});

function RootShell({ children }: { children?: unknown }) @{
  <Html lang="en">
    <Head>
      <script dangerouslySetInnerHTML={{ __html: THEME_BOOT_SCRIPT }} />
      <script dangerouslySetInnerHTML={{ __html: FONT_WARM_SCRIPT }} />
      <meta charSet="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
      <link
        rel="preload"
        href="/fonts/sora-latin.woff2"
        as="font"
        type="font/woff2"
        crossOrigin="anonymous"
      />
      <link
        rel="preload"
        href="/fonts/source-sans-3-latin.woff2"
        as="font"
        type="font/woff2"
        crossOrigin="anonymous"
      />
      <link rel="icon" href="/favicon.ico" sizes="any" />
      <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png" />
      <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16.png" />
      <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
      <link rel="manifest" href="/manifest.webmanifest" />
      <meta name="theme-color" content="#dfe8f0" media="(prefers-color-scheme: light)" />
      <meta name="theme-color" content="#07080a" media="(prefers-color-scheme: dark)" />
      <meta name="apple-mobile-web-app-title" content="Octanest" />
      <HeadContent />
    </Head>
    <Body>
      {children}
      <Scripts />
      <script dangerouslySetInnerHTML={{ __html: SW_REGISTER_SCRIPT }} />
    </Body>
  </Html>
}

function RootComponent() @{
  const data = useLoaderData({ from: "__root__" }) as RootLoaderData | undefined;
  const queryClient = getQueryClient();

  <QueryClientProvider client={queryClient}>
    <HydrationBoundary state={data?.dehydratedState}>
      <AppToaster>
        <div className="flex min-h-screen flex-col">
          <SiteHeader />
          <main className="octanest-main flex-1">
            <Outlet />
          </main>
          <SiteFooter />
          <VerifyBanner />
        </div>
      </AppToaster>
    </HydrationBoundary>
  </QueryClientProvider>
}
