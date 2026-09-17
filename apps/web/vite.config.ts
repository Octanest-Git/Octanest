import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { tanstackStart } from "@octanejs/tanstack-start/plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import {
  fixTypeOnlyImports,
  RECHARTS_TYPE_ONLY_IMPORT_FIX,
} from "./vite-plugins/fix-type-only-imports.ts";
import { webHealthPlugin } from "./vite-plugins/web-health.ts";

// NOTE (03-05): vite-plugin-pwa was evaluated here but does not emit a service
// worker under this Vite 8 / @octanejs/tanstack-start multi-environment build
// (the manifest virtual module resolves, but the workbox-build closeBundle
// hook never fires — no sw.js is produced, silently). Per the 03-05 plan's
// documented fallback, the manifest and service worker are hand-authored as
// static files in apps/web/public/ instead (manifest.webmanifest, sw.js).
// See .planning/phases/03-brand-shell-theme/03-05-SUMMARY.md for details.
const rootDir = path.dirname(fileURLToPath(import.meta.url));
const apiProxyTarget =
  process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") || "http://127.0.0.1:8080";

export default defineConfig({
  plugins: [
    // Before Start/proxy: Compose + Dockerfile probe `/health` with Octanest-Health-Probe.
    webHealthPlugin(),
    fixTypeOnlyImports(RECHARTS_TYPE_ONLY_IMPORT_FIX),
    tanstackStart({
      // Keep colocated *.integration.test.* / *.unit.test.* out of the route tree
      // (avoids noisy warnings and extra SSR work during stack e2e).
      // Must live under `router` — top-level keys are stripped by Start's schema.
      router: {
        routeFileIgnorePattern: "\\.(test|spec)\\.",
      },
    }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "./src"),
      // Published attr-accept "module" build is fake ESM (`exports` in browser →
      // hydration abort). Bun also nests it so optimizeDeps.include cannot resolve.
      "attr-accept": path.resolve(rootDir, "./src/shims/attr-accept.ts"),
    },
  },
  server: {
    port: 3000,
    proxy: {
      "/api/rpc/ws": { target: apiProxyTarget.replace(/^http/, "ws"), ws: true },
      "/api/rpc": { target: apiProxyTarget, changeOrigin: true },
      "/api/auth": { target: apiProxyTarget, changeOrigin: true },
      "/api/user": { target: apiProxyTarget, changeOrigin: true },
      "/api/repos": { target: apiProxyTarget, changeOrigin: true },
      "/api/admin": { target: apiProxyTarget, changeOrigin: true },
      "/api/releases": { target: apiProxyTarget, changeOrigin: true },
      "/uploads": { target: apiProxyTarget, changeOrigin: true },
      // `/health` is owned by webHealthPlugin (header-gated). Public API /health
      // remains via Traefik → api in Compose; do not proxy here.
      // Phase 20 package registry (D-PKG-01) — same-host path prefixes → API
      "/v2": { target: apiProxyTarget, changeOrigin: true },
      "/npm": { target: apiProxyTarget, changeOrigin: true },
      "/generic": { target: apiProxyTarget, changeOrigin: true },
    },
  },
});
