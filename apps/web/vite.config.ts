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
      "/api/rpc/ws": { target: "ws://127.0.0.1:8080", ws: true },
      "/api/rpc": { target: "http://127.0.0.1:8080", changeOrigin: true },
      "/api/auth": { target: "http://127.0.0.1:8080", changeOrigin: true },
      "/health": { target: "http://127.0.0.1:8080", changeOrigin: true },
    },
  },
});
