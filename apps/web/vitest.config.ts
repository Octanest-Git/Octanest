import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const stackEnabled = process.env.E2E_STACK === "1";

/**
 * Vitest projects:
 * - unit / integration / e2e-component: always on in `bun run test`
 * - e2e-stack (+ browser): only when E2E_STACK=1 (`make test-e2e-stack`)
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
          name: "e2e-component",
          include: ["e2e/component/**/*.e2e.test.{ts,tsx}"],
          setupFiles: ["./e2e/component/setup.ts"],
          browser: {
            enabled: true,
            provider: playwright(),
            headless: true,
            instances: [{ browser: "chromium" }],
          },
        },
      },
      ...(stackEnabled
        ? [
            {
              extends: true as const,
              test: {
                name: "e2e-stack",
                environment: "node" as const,
                include: ["e2e/stack/**/*.stack.test.ts"],
                fileParallelism: false,
                setupFiles: ["./e2e/stack/setup.ts"],
                testTimeout: 60_000,
              },
            },
            {
              extends: true as const,
              test: {
                name: "e2e-stack-browser",
                include: ["e2e/stack-browser/**/*.stack.browser.test.{ts,tsx}"],
                setupFiles: ["./e2e/stack-browser/setup.ts"],
                fileParallelism: false,
                testTimeout: 60_000,
                browser: {
                  enabled: true,
                  provider: playwright(),
                  headless: true,
                  instances: [{ browser: "chromium" as const }],
                },
              },
            },
          ]
        : []),
    ],
  },
});
