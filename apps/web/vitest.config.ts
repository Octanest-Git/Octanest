import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

const rootDir = path.dirname(fileURLToPath(import.meta.url));

/**
 * Vitest projects:
 * - unit: pure Node (libs, helpers)
 * - integration: happy-dom + Testing Library (component contracts)
 * - e2e: real Chromium via @vitest/browser-playwright (browser flows)
 *
 * Deliberately omits @octanejs/tanstack-start so the test Vite graph stays light.
 */
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "./src"),
    },
  },
  test: {
    globals: false,
    projects: [
      {
        extends: true,
        test: {
          name: "unit",
          environment: "node",
          include: ["src/**/*.unit.test.ts"],
        },
      },
      {
        extends: true,
        test: {
          name: "integration",
          environment: "happy-dom",
          include: ["src/**/*.integration.test.{ts,tsx}"],
          setupFiles: ["./src/test/setup-integration.ts"],
        },
      },
      {
        extends: true,
        test: {
          name: "e2e",
          include: ["e2e/**/*.e2e.test.{ts,tsx}"],
          setupFiles: ["./e2e/setup.ts"],
          browser: {
            enabled: true,
            provider: playwright(),
            headless: true,
            instances: [{ browser: "chromium" }],
          },
        },
      },
    ],
  },
});
