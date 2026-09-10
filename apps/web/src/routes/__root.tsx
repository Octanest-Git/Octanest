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
import { VerifyBanner } from "@/components/verify-banner";
import { THEME_BOOT_SCRIPT } from "@/lib/theme";
import "@/styles.css";

// Static literal, no interpolation (T-03-19) — registers the hand-authored
// assets-only service worker (see apps/web/public/sw.js, 03-05-SUMMARY.md).
const SW_REGISTER_SCRIPT =
  '(function(){if("serviceWorker" in navigator){window.addEventListener("load",' +
  'function(){navigator.serviceWorker.register("/sw.js").catch(function(){});});}})();';

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
        <script dangerouslySetInnerHTML={{ __html: THEME_BOOT_SCRIPT }} />
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
  );
}

function RootComponent() {
  return (
    <div className="flex min-h-screen flex-col">
      <SiteHeader />
      <VerifyBanner />
      <main className="octanest-main flex-1">
        <Outlet />
      </main>
      <SiteFooter />
    </div>
  );
}
