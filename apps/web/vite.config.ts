import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { tanstackStart } from "@octanejs/tanstack-start/plugin/vite";
import tailwindcss from "@tailwindcss/vite";

// NOTE (03-05): vite-plugin-pwa was evaluated here but does not emit a service
// worker under this Vite 8 / @octanejs/tanstack-start multi-environment build
// (the manifest virtual module resolves, but the workbox-build closeBundle
// hook never fires — no sw.js is produced, silently). Per the 03-05 plan's
// documented fallback, the manifest and service worker are hand-authored as
// static files in apps/web/public/ instead (manifest.webmanifest, sw.js).
// See .planning/phases/03-brand-shell-theme/03-05-SUMMARY.md for details.
const rootDir = path.dirname(fileURLToPath(import.meta.url));
const apiProxyTarget =
  process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") ||
  "http://127.0.0.1:8080";

export default defineConfig({
  plugins: [tanstackStart(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "./src"),
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
      "/api/releases": { target: apiProxyTarget, changeOrigin: true },
      "/uploads": { target: apiProxyTarget, changeOrigin: true },
      "/health": { target: apiProxyTarget, changeOrigin: true },
      // Phase 20 package registry (D-PKG-01) — same-host path prefixes → API
      "/v2": { target: apiProxyTarget, changeOrigin: true },
      "/npm": { target: apiProxyTarget, changeOrigin: true },
      "/generic": { target: apiProxyTarget, changeOrigin: true },
    },
  },
});
